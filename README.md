# 介绍

学习rust项目

资源链接
- [B站资源链接](https://www.bilibili.com/video/BV1KB4y177kz/?spm_id_from=333.788.player.switch&vd_source=d6911fee4334160cf6dbc744c6a0e09f)
- [github](https://github.com/tyrchen/reservation/tree/master)



# Start
安装protoc, 安装命令

```bash
# 需要先安装protobuf
sudo apt update
sudo apt install python3-pip
pip install protobuf
# 安装protoc
sudo apt install protobuf-compiler

# 安装sqlx-cli
# wsl ubuntu
sudo apt update && sudo apt install -y pkg-config libssl-dev
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

启动命令
```bash
# 需要先启动 docker daemon 
sudo service docker start
```