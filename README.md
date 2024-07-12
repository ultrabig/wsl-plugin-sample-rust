# WSL Plugin Sample in Rust
This is a Rust port of [Microsoft/wsl-plugin-sample](https://github.com/microsoft/wsl-plugin-sample).

## Usage
1. [Install Rust](https://learn.microsoft.com/en-us/windows/dev-environment/rust/setup)
2. [Install Clang](https://rust-lang.github.io/rust-bindgen/requirements.html)
3. Download the WSL Plugin header from [Nuget](https://www.nuget.org/packages/Microsoft.WSL.PluginApi)
    * Unzip the .nupkg file and copy `build\native\include\WslPluginApi.h` to `wsl-plugin-api-sys\include\WslPluginApi.h`
4. `cargo build`
5. Open a Visual Studio Developer Command Prompt as administrator and sign the plugin
    ```bat
    cd path\to\wsl-plugin-sample-rust
    powershell .\scripts\sign-plugin.ps1 -Trust
    ```
6. Register the plugin
    ```bat
    .\scripts\register.cmd
    ```
7. Stop the wslservice to load the plugin
    ```bat
    .\scripts\stop-wsl.cmd
    ```
8. Once loaded, open `C:\wsl-plugin-demo.txt` to see the plugin output
