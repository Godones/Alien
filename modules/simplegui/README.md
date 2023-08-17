# simplegui
一个使用`embedded graphics`框架的简单gui实现。`embedded graphics`提供了一套简单的图形绘制接口，可以在不同的平台上实现。
但是这些图形绘制功能比较基础，并且没有窗口管理，事件传播等机制。因此其适合用作底层库来搭建上层的窗口管理系统。

## 实现
目前我们只根据其提供的基本绘制功能添加一些简单的部件支持：

- bar
- button
- edit
- icon
- image
- label
- graphic
- panel

这些基本部件可以组合成复杂的窗口，但是这些窗口并没有窗口管理功能，因此后续我们需要一个窗口管理器来管理这些窗口。

基于这些部件，我们实现了几个比较复杂的部件:

- desktop
- terminal
- snake game


## 待完成
这些基本的窗口虽然可以显示，但是我们还缺少事件的传递机制。

## 资料

https://samsartor.com/guis-1/
https://samsartor.com/guis-2/ 

https://samsartor.com/guis-3/