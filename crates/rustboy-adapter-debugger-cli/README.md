# rustboy-adapter-debugger-cli

`rustboy-adapter-debugger-cli` is the planned terminal adapter for the Rustboy debugger. It will translate text commands into debugger use cases from `rustboy-debugger`; it will not contain breakpoint or machine-state policy itself.

The first version is expected to provide an interactive REPL with commands for stepping, continuing, inspecting registers and memory, adding breakpoints, and viewing a short execution trace. This adapter is currently scaffolding only.
