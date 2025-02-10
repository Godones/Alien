# 有状态域的动态更新机制

有状态域与无状态域不一样，在前面的实现中，对于无状态域，我们使用RCU机制对其完成了动态更新的机制，但是有状态的域无法直接套用rcu

- 允许在新旧状态运行的程序使用rcu，因为rcu在更新期间允许读者继续使用旧数据
- 在有状态的域中，这些进入的读者可能会导致域的状态发生改变，而域的更新者在更新过程中读取旧数据

根本原因在于：有状态的域应该保证在域更新前后保证状态的一致性

如果把这里的RCU直接换成读写锁 ==> 会导致较大的开销

## 同步机制

### 条件判断

```rust
fn call(){
	if (not in updating){
        return domain.call()
    }
    let guard = Lock.lock();
    domain.call();
    drop(guard);
}
```

- 每个域代理有一个**更新标志**
- 在非更新时间，域代理直接调用域的具体实现
- 当需要更新这个域的时候
  - 首先修改**更新标志**
  - 后续如果有新的调用者，其会尝试获取读锁并进入域中
  - 更新者作为写者尝试获取写锁进入域中（独占性）
  - 这样就可以保证域的更新过程中状态的一致性
  - <font color = red> 这里产生了一个问题：如果在修改指令前已经有读者进入域中，写者需要确保这些读者也离开了域，而后续的读者可以凭借写锁的获得就知道他们已经离开了。  </font>
    - 只需要在进出域前增加和减少计数器即可

### jmp to new impl

作为条件判断的优化，在非更新期间，只有一条空指令的开销。

参考linux系统中 static key机制

```rust
fn call(){
	asm!("nop"); // ->asm!("jmp to new impl")
    domain.call();
}
fn new_call(domain){
    let guard = Lock.lock();
    domain.call();
    drop(guard);
}

```

1. 在域的正常运行期间，这个`nop` 指令不做如何事情
2. 当需要更新这个域的时候，将这个指令修改为一个`jmp` 指令，跳转到一个需要获取锁的实现中



### 并行处理

在使用条件判断的情况下，多核场景下，可能会出现的一种情况：

非更新线程访问这个域，更新线程需要更新这个域。非更新线程首先读取标志，发现此时没有处于更新状态，因此其选择走无锁的路径。其需要增加计数器表示进入域中，但在增加操作前，

此时更新线程更新了这个标志。更新线程尝试获取写锁并拿到写锁，然后统计此时是否仍然有非更新线程位于域中。

这个操作发生在了非更新者增加计数器操作前，导致更新线程判断此时没有其它线程位于域中， 其开始回收域的资源并完成域的更新操作。

非更新线程此时可能拿到旧的域，并且访问旧的域，但此时旧的域的资源已经被释放，或者更新者正在进行访问，这破坏了域的独占性。



这里的核心问题是：非更新线程从判断域是否在更新状态到拿到递增计数器并拿到域对象不是一个原子的过程。

为了解决这个问题，我们利用了rcu实现的一个功能来进行同步：

```rust
self.in_updating
            .store(true, core::sync::atomic::Ordering::Relaxed);

// why we need to synchronize_sched here?
synchronize_sched();

// stage2: get the write lock and wait for all readers to finish
let w_lock = self.lock.write();
while self.all_counter() > 0 {
    println!("Wait for all reader to finish");
    yield_now();
}
```

在更新线程更新了标志之后，我们不能立即判断所有非更新线程是否已经离开域，而是首先调用`synchronize_sched` ，这个函数会同步所有核心上的线程，其具体的表现就是使得当前线程在每个核心上都经历了一次上下文切换，从而可以推断出其它核心上的线程一定已经完成了计数和获取域对象的过程，因为这个过程不会被打断，也就不会发生上下文切换。

当更新线程在所有核心上经历一次上下文切换后，其就可以正确去判断是否有其它线程位于域中了。

这里没有将`synchronize_sched`放在获取写锁之后，是为了使得非更新线程不阻塞在读锁上，因为在更新线程同步期间仍然有非更新线程可能访问这个域。



## 状态保存和恢复

**核心见解：利用rust的自定义分配器来保存需要在新旧域中使用的数据或状态**

对于宏内核来说，各个子系统可能包含了大量的状态信息或数据。造成状态溢出。要想对一个子系统进行更新，除了完成新旧模块的更新之外，其内部的数据也要正确进行迁移，否则其对用户程序提供的服务就会被中断。

之前的研究工作主要有几种方案：

1. 对微内核的用户态服务进行重启，但简单的重启导致内部的状态全部丢失，无法继续恢复之前的服务
2. [CuriOS: improving reliability through operating system structure](https://dl.acm.org/doi/10.5555/1855741.1855746)  对第一种方案更近一步，提出在服务端一个单独的内存区域存储客户端的相关信息，并保持这个区域对客户端的不可见性，当服务端被重启，其可以重新使用这个内存区域的内容。 这种方案的一个缺点是需要对服务端进行较大的侵入式修改，因为数据需要存放在新的区域了
3. Thesues对状态保存的见解则是将状态全部存放在客户端任务中，也就是说，由任务保存所有的状态。这种方案只使用与其单一地址空间，单一特权级架构下，因为宏内核中，用户任务只能通过syscall来交互，这个接口并不允许传递意义丰富的数据，内核不得不维护这些数据和用户任务的关系。

我们的方法与这二种类似，但是更多地利用了现代语言提供的机制。

### Allocator API

rust的堆数据结构在默认情况下向全局的堆分配器分配内存空间。同时也为这些数据结构提供了第二种选择，也就是自定义堆分配器。在指定自定义堆分配器的情况下，堆数据结构从这个堆上分配内存空间而不是全局堆。

我们可以利用这个性质，将有状态域中需要传输到新的域中的数据保存在自定义堆中，而其它非更新数据依然使用正常的全局堆分配器进行分配。在更新期间，域的资源被正确回收，包括其使用的全局堆资源，而其在自定义堆中保存的数据则被保留下来，更新之后的域将直接继承这些数据。

allocator api的使用方式如下所示:

```rust
#[derive(Clone)]
struct MyAllocator;

unsafe impl Allocator for MyAllocator{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        println!("allocate from MyAllocator, size: {}",layout.size());
        System.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        println!("deallocate from MyAllocator, size: {}",layout.size());
        System.deallocate(ptr, layout)
    }
}
let mut v = Vec::new_in(MyAllocator);
v.push(1);
v.reserve(100);
```



### Storage

有了存储数据的方式和位置，还需要解决的一个问题是新的域如何找到旧的域存放的数据。**这里我们的解决方案很简单: 用一个键值对表存储这个数据结构的引用**。这个表提供的接口如下所示:

```rust
pub trait DomainDataStorage: Send + Sync {
    fn insert(
        &self,
        key: &str,
        value: Box<Arc<dyn Any + Send + Sync, DataStorageHeap>, DataStorageHeap>,
    ) -> Option<Box<Arc<dyn Any + Send + Sync, DataStorageHeap>, DataStorageHeap>>;
    fn get(&self, key: &str) -> Option<Arc<dyn Any + Send + Sync, DataStorageHeap>>;
}
```

对于域实现这来说，其通过下面三个接口创建和索引数据:

```rust
pub fn insert_data<T: Any + Send + Sync>(
        key: &str,
        value: T,
    ) -> Option<Arc<T, DataStorageHeap>>

pub fn get_data<T: Any + Send + Sync>(key: &str) -> Option<Arc<T, DataStorageHeap>>
pub fn get_or_insert_with_data<T: Any + Send + Sync, F: FnOnce() -> T>(
        key: &str,
        f: F,
) -> Arc<T, DataStorageHeap>
```

1. 域在内部创建需要更新同步的数据时，数据需要保存在自定义堆上
2. 域通过一个键值对将数据保存在表中，同时会返回一个引用
3. 域通过引用操作这个数据，而对数据的增删改查都发生在自定义堆上

这里的一个前提是： **我们认为对实施域更新的开发者来说，其知晓旧的域保存数据所用的索引，从而可以在新的域中索引之前的数据**。



## 实现

由于每个域都是独立编译的，而这个存储表应该是统一的，持久性的。为了做到这一点，由TCB在加载域的时候的时候注入这个表，同时注入的还有自定义分配器。因为这两个数据结构都由接口定义，因此其做法基本与之前的私有堆和共享堆的实现是类似的。我们只需要在域初始化时注入这些资源即可。

```rust
fn main(
    sys: &'static dyn CoreFunction,
    domain_id: u64,
    shared_heap: &'static dyn SharedHeapAlloc,
    storage_arg: StorageArg,
) -> Box<dyn BufUartDomain> {
    ....
    // init storage
    let StorageArg { allocator, storage } = storage_arg;
    storage::init_database(storage);
    storage::init_data_allocator(allocator);
}
```

### 调度域

以调度器为例，我们看域如何保存和恢复这些需要同步的更新数据：

```rust
type __TaskList = Mutex<VecDeque<DBox<TaskSchedulingInfo>, DataStorageHeap>>;
type TaskList = Arc<__TaskList, DataStorageHeap>;
#[derive(Debug)]
pub struct CustomFiFoScheduler {
    tasks: TaskList,
}

impl CustomFiFoScheduler {
    pub fn new() -> Self {
        let task_list = storage::get_or_insert_with_data::<__TaskList, _>("tasks", || {
            __TaskList::new(VecDeque::new_in(DataStorageHeap))
        });
        Self { tasks: task_list }
    }
}
```

调度器的基本实现中，在初始化时，我们首先在自定义堆上创建了数据，同时获得了其引用。

在一个新的 调度器域实现中，可以产生如下的代码:

```rust
#[derive(Debug)]
pub struct RandomScheduler {
    tasks: TaskList,
}

impl RandomScheduler {
    pub fn new() -> Self {
        println!("RandomScheduler: new");
        let task_list = storage::get_data::<__TaskList>("tasks").unwrap();
        let len = task_list.lock().len();
    }
}
```

因为我们指导这个新的域被加载后，其替换的前一个域已经创建了一个任务列表，因此我们这里可以直接索引原来的任务列表。



### vfs域实现

vfs域会负责初始化其它文件系统域，比如devfs/procfs/fatfs等，在初始化的时候，这些域会返回其根文件的`InodeID`, 在vfs域内，会根据这个`InodeID`创建一系列对象，因为域的划分，每个域内部的对象无法直接传递。这些对象会去实现对应的`trait` , 相当于在vfs域内部创建一个和文件系统域内对象对应的一个虚拟对象，后续这个虚拟对象会直接使用其拥有的`InodeID` 与文件系统域交互。

vfs还会保存打开的文件，vfs内部也是用`InodeID`来维护这个信息，这里的`InodeID` 和上面的不能混淆。

综上，如果想要在vfs域做更新，其保存的状态信息应该设计如下：

#### vfs保存的打开的文件

```rust
static VFS_MAP: RwLock<BTreeMap<InodeID, Arc<dyn File>>> = RwLock::new(BTreeMap::new());
static INODE_ID: AtomicU64 = AtomicU64::new(4);
```

vfs_map存储`InodeID` 到 `Arc<dyn File>` 的映射，`InodeID`保存了一个计数器，用来分配`InodeID`。

在更新vfs域后，我们应该仍然需要这两个信息，但是`Arc<dyn File>`  不能直接放在自定义堆上，因为我们对接口的相关定义是没有将参数放在自定义堆上的。因此这里我们应该存储一个 `Arc<dyn File>` 内部对应其它文件系统的标识符`InodeID`以及其它元数据信息。

```rust
struct FsShimInode {
    ino: InodeID,
    fs_domain: Arc<dyn FsDomain>,
    sb: Mutex<Option<Weak<dyn VfsSuperBlock>>>,
}
```

可以直接从这里找到`Arc<dyn File>`内部真正的`InodeID`, 从这个信息中我们可以直接重建`FsShimInode` , 进而重建`Arc<dyn File>`。 除了这个`InodeID`信息外，还需要一些其它信息:

```rust
pub struct KernelFile {
    pos: Mutex<u64>,
    open_flag: Mutex<OpenFlags>,
    dentry: Arc<dyn VfsDentry>,
}
```

这里的`pos`  和 `open_flag`是我们需要的。

对上面的分析，我们可以将vfs保存这些信息的方法进行修改，使得这些我们需要的信息被保存在自定义堆上:

```rust
type DeviceIdManagerType = Arc<Mutex<DeviceIdManager>,DataStorageHeap>;
static DEVICE_ID_MANAGER: Lazy<DeviceIdManagerType> = Lazy::new(||{
    let res = storage::get_or_insert_with_data("device_id_manager", || {
        Mutex::new(DeviceIdManager::new())
    });
    res
});
```



```rust
pub type KMeta = Arc<Mutex<KernelFileMeta>, DataStorageHeap>;
#[derive(Debug)]
pub struct KernelFileMeta {
    pos: u64,
    open_flag: OpenFlags,
    real_inode_id: u64,
}
```



```rust
static INODE_ID: Lazy<Arc<AtomicU64, DataStorageHeap>> = Lazy::new(|| {
    let id = storage::get_or_insert_with_data("inode_id", || AtomicU64::new(4));
    id
});

static VFS_INIT: Lazy<Arc<AtomicBool, DataStorageHeap>> = Lazy::new(|| {
    let res = storage::get_or_insert_with_data("vfs_init", || AtomicBool::new(false));
    res
});
```



```rust
type DataType = Arc<Mutex<BTreeMap<InodeID, (), DataStorageHeap>>, DataStorageHeap>;
static VFS_MAP_SHADOW: Lazy<DataType> = Lazy::new(|| {
    let res = storage::get_or_insert_with_data("inode2inode", || {
        Mutex::new(BTreeMap::new_in(DataStorageHeap))
    });
    res
});
```



### 注意

#### HACK

rust的 `dyn Any` 类型的转换会导致当旧域的代码和数据被删除时在新的域中进行转换时造成错误，似乎是因为编译器会将类型的信息保存在一个位置，然后在判断类型是否相等时去查找这个信息。因为信息已经被删除，这会触发`LoadPageFault`。

解决方法：

1. 手工保存对应key的类型信息，在转换时对比： 这带来了存储的开销，即使其类型信息原本就存在了
   1. 直接强制转换，由调用者确保类型的正确性
2. https://crates.io/crates/dyn-dyn 



- 保证需要更新的数据结构在新的域中具有相同的定义，否则rust编译器会将其当作两个不同的数据对待，这会导致类型转换错误：将数据的定义放在共用的lib中
- 



## 资源回收的修改

在有状态域发生更新时，如果直接回收其内部分配的资源可能造成问题。因为自定义堆上可能保存了指向共享堆的数据。更新堆上的数据通常由域在发生域间通信时创建，并可能被域保存，即使这些共享数据最终会被回收。

在之前对无状态域的处理中，我们直接回收了域中所有的资源，这里面通常不包含共享堆上数据(我们对共享堆上的数据定义本来就是如此)

也就是说，对于共享堆上的资源的释放，主要包含几种情况：

1. 当域发生损坏，我们重启域的时候
   1. 如果域是无状态的，则传入的被保存的共享堆资源/自行创建的共享堆资源被一并释放
   2. 如果域是有状态的
      1. 在未保存这个共享堆数据前崩溃:   应该被释放，重启者应该会再次传入新的数据
      2. 在访问这个共享堆数据时崩溃 :  应该被释放， 重启者应该会再次传入新的数据
      3. 在保存了这个共享堆数据时崩溃：被保留，重启者应该会再次传入新的数据？如何解决这个问题？
2. 正常对域进行更新升级的时候
   1. 如果域是无状态的，则传入的被保存的共享堆资源/自行创建的共享堆资源被一并释放
   2. 如果域是有状态的，则传入的被保存的共享堆资源/自行创建的共享堆资源被保留

目前对域重启机制仍然不完善，不好处理资源释放的问题。

## 优势

1. 直接使用语言提供的机制，不需要大量的修改
2. 适用性广泛



## Todo

- [x] 每个域有自己的表，避免不同域之间使用同一个键产生冲突
- [x] 添加删除key映射的接口，新的域可能拿到旧的数据后不会再使用旧的域的数据，其会将旧域创建的映射删除
