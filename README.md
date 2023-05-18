# 💜 tf-doom

Entertaining Terraform chaos engineering, destroy resource by killing DOOM ennemies.

This is a Rust fork of [kubedoom](https://github.com/storax/kubedoom), forked from [dockerdoom](https://github.com/gideonred/dockerdoom), forked from  **`psdoom`**. 

Technically, you could run this project outside a Docker container but the project was especially designed to run in one

## 📖 How to build and run ?

1. Install the system dependencies
    - `docker`

## 🖼️ Screenshot

![In game](./assets/in-game.png)

## ℹ️ Usage

An example with the Terraform project in `./test`. Feel free to **`terraform apply`** before or after running the Docker container, both will work.

The Terraform project directory must be bound at `/tf` inside the container (like below).

```bash
docker run \
    -itd \
    --rm=true \
    --name tf-doom \
    -p 5900:5900 \
    -v $PWD/test:/tf \
    b0thr34l/dockerdoom:1.0
```

Now you can play DOOM through a VNC client. Example with `vnclient`:

```bash
vncviewer viewer localhost:5900
```

## 🔎 Cheat codes

There are some useful cheat codes in-game:
- **`idkfa`**: Get a weapon on slot 5
- **`idspispopd`**: No clip (useful to reach the mobs)
