reg.exe add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss\Plugins" /v sample-plugin /d %cd%\target\debug\wsl_plugin_sample.dll /t reg_sz /f
