# Arhiv for Android

The Android application is a Java WebView wrapper around the local Rust Arhiv
server. It uses JNI to start the server and Android Keystore plus device
authentication to protect a cached storage master key.

## Runtime requirements

- Android 11 (API 30) or newer;
- Android System WebView major version 111 or newer; and
- all-files access so Arhiv can use `<externalStorage>/Arhiv`.

## Build requirements

- Android Studio with Android SDK and NDK;
- JDK 17;
- Rust targets for the device architectures being built;
- `cargo-ndk`; and
- a signing keystore plus `arhiv-android/keystore.properties`.

Install the Rust targets used by the repository recipes:

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install --locked cargo-ndk
```

The Android `minSdk` in `app/build.gradle` and `android_platform_version` in the
root `justfile` are both API 30 and must remain aligned.

## Signing configuration

The Gradle build reads signing configuration during project setup, including
for local debug builds. Generate a development keystore if you do not already
have one:

```sh
keytool -genkeypair \
  -alias Arhiv \
  -keyalg RSA -keysize 2048 \
  -validity 10000 \
  -keystore release.keystore \
  -dname "CN=Your Name, OU=Your Org, O=Your Company, L=City, ST=State, C=US" \
  -storepass YOUR_STORE_PASS \
  -keypass YOUR_KEY_PASS
```

Place `release.keystore` in `arhiv-android/` and create
`arhiv-android/keystore.properties`:

```properties
storeFile=../release.keystore
storePassword=YOUR_STORE_PASS
keyAlias=Arhiv
keyPassword=YOUR_KEY_PASS
```

Do not commit either file.

## Build and install

Run recipes from the repository root.

Build native libraries for a physical arm64 device and assemble a debug APK:

```sh
just build-android-libs build-android-app
```

For the x86_64 emulator, use:

```sh
just build-android-emulator-libs build-android-app
```

Install the debug application with `just install-android-app`.

Build the signed release APK with:

```sh
just prod-build-android-libs prod-build-android-app
```

The release recipe writes `arhiv.apk` at the repository root. Install it with
`just install-prod-android-app`.

Run Android unit tests with `just test-android-app`.

## WebView debugging

1. Connect a device with developer mode and USB debugging enabled.
2. Run a debug build of Arhiv.
3. Open `chrome://inspect` in desktop Chrome.
4. Select the Arhiv WebView under **Remote Target**.
