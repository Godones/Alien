#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <unistd.h>

void *writer_thread(void *arg) {
  int efd = *(int *)arg;
  sleep(1); // 模拟延迟
  uint64_t val = 1;
  if (write(efd, &val, sizeof(val)) != sizeof(val)) {
    perror("write to eventfd");
  } else {
    printf("[Writer] Wrote to eventfd\n");
  }
  return NULL;
}

int main() {
  int efd = eventfd(0, 0);
  if (efd == -1) {
    perror("eventfd");
    exit(EXIT_FAILURE);
  } else {
    printf("[Main] Created eventfd: %d\n", efd);
  }

  int epfd = epoll_create1(0);
  if (epfd == -1) {
    perror("epoll_create1");
    exit(EXIT_FAILURE);
  }

  struct epoll_event ev = {0};
  ev.events = EPOLLIN;
  ev.data.fd = efd;

  if (epoll_ctl(epfd, EPOLL_CTL_ADD, efd, &ev) == -1) {
    perror("epoll_ctl");
    exit(EXIT_FAILURE);
  }

  pthread_t thread;
  if (pthread_create(&thread, NULL, writer_thread, &efd) != 0) {
    perror("pthread_create");
    exit(EXIT_FAILURE);
  }

  struct epoll_event events[1];
  printf("[Main] Waiting for event...\n");
  int nfds = epoll_wait(epfd, events, 1, -1); // 阻塞等待事件
  if (nfds == -1) {
    perror("epoll_wait");
    exit(EXIT_FAILURE);
  }

  printf("[Main] epoll_wait returned %d\n", nfds);

  if (events[0].data.fd == efd) {
    uint64_t val;
    if (read(efd, &val, sizeof(val)) != sizeof(val)) {
      perror("read from eventfd");
    } else {
      printf("[Main] Received eventfd value: %lu\n", val);
    }
  } else {
    printf("[Main] Unexpected file descriptor %d\n", events[0].data.fd);
  }

  pthread_join(thread, NULL);
  close(efd);
  close(epfd);
  printf("test_epoll finished\n");
  return 0;
}
