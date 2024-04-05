# Isolation in os subsystem



## 1. Introduction



## 2. Implementation



## 3. Create a new Domain

1. run cargo command

   ```
   cargo domain new --name {domain_name}
   ```

2. choose the domain type

   ```
   1. Common
   2. Fs
   3. Driver
   ```
3. input the domain interface name
   
   ```
   {interface_name}
   ```
4. build one domain
   
   ```
    cargo domain build --name {domain_name}
    ```