# eyMate

[![pre-commit](https://github.com/LDprg/eyMate/actions/workflows/pre-commit.yml/badge.svg)](https://github.com/LDprg/eyMate/actions/workflows/pre-commit.yml)
[![Rust](https://github.com/LDprg/eyMate/actions/workflows/rust.yml/badge.svg)](https://github.com/LDprg/eyMate/actions/workflows/rust.yml)

Linux face detection similar to Windows Hello and Howdy written in rust.

> [!NOTE]
> This project is very much new. It should would, but expect some stuff not to work as expected.

## How to Install

EyMate provides following package formats (and build commands):

> [!NOTE]
> Only the pacman is offically maintained by me (I use Arch btw.), other (rpm and dpkg) might not work.

- pacman: `cargo aur`
- rpm: `cargo build && cargo generate-rpm`
- dpkg: `cargo deb`

The packages will be in a subfolder from 'targets'.

Alternative download the packages from the github releases.

### PAM configuration

> [!NOTE]
> Without the pam entries eyMate won't work. PAM config files are for specific auth types.
> You will most likely change Following entries:
>
> - sudo (sudo prompts)
> - kde (KDE screenunlock)
> - polkit-1 (GUI prompts)

> [!WARN]
> Adding eyMate login to any display manager (sddm, gdm, etc.) is NOT RECOMMENDED!
> It will cause bugs (like with kwallet) and will make you system very unsecure!

Add following to pam config at the top of the file:

```
auth        sufficient      pam_unix.so  try_first_pass likeauth nullok
auth        sufficient      libpam_eymate.so
```

This will cause you login prompt to try face detection whenever you type in a wrong password.
Just hit enter when you want to use face detection.

Alternative you could do it the other way around:

```
auth        sufficient      libpam_eymate.so
auth        sufficient      pam_unix.so likeauth nullok
```

This will first eyMate to do the auth and if it fails run a normal password login.

> [!NOTE]
> Be aware that the second method will instantly cause the face detection to run and might be a security issue (unintentional logins).

### Manual Install

Build project with `cargo build --release`.

You will find the executable and pam lib in the 'target/release' folder.
Copy the 'eymate' executable to '/usr/bin/' to install globally (optional, only needed for cli).
Copy the 'libpam_eymate.so' libary to '/usr/lib/security/' (required for pam auth).
Copy from the 'prebuild' folder the 'vggface2.pt' file to '/etc/eymate/' (required by cli and pam auth).

#### Dependencies

You need to have following installed:

- pytorch
- torchvision
- python-gobject
- opencv
- pam

- facenet (git clone into project root, only for building vgg2face.pt)

#### Build vgg2face model

Just follow these steps:

- install dependencies
- run build_model.py
- copy created model ('vgg2face.pt') to '/etc/eymate/'

## How to use

### Configure

Configure eyMate with the config file found in '/etc/eymate/config.toml'

```toml
[video]
mode = "IR"         # Mode IR (ir cam) or RGB (normal cam)
device_rgb = 0      # Normal cam number /dev/video0
device_ir = 2       # IR cam number /dev/video2

[detection]
min_similarity_rgb = 0.7    # Min Sinilarity for face detection (lower means easier detection but worse for security)
min_similarity_ir = 0.9     # Same as above, but for ir cam
min_brightness_rgb = 50.0   # Min brightness for face detection (to prevent bad frames/detection)
min_brightness_ir = 10.0    # Same as above, but for ir cam
retries = 10                # Amount of retries befor auth fails
```

### Add User

Add user with following command:

```bash
eymate add <user>
```

### Test User

Test user with following command:

```bash
eymate test <user>
```

User must be added first, otherwise test will fail.

## FAQ

### IR Cam not working / all black

You might want to install and enable [linux-enable-ir-emitter](https://github.com/EmixamPP/linux-enable-ir-emitter), so your IR emitter turns automatically on.

### Can I contribute

Of course feel free to open issues and pr's at any time. However note that you should open a issue for bigger features first, to prevent work duplication and misconceptions.

## Disclaimer

> [!WARN]
> Use this software at your own risk! It is most certainly bad for security, so don't expect your system to be safe with it.
> I am not responsible for any damage done or cause by this software!
