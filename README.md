# Code for my research on code mobility using WebAssembly
### Structure:
- AI-history: Contains a cache application that captures the history of the prompts made by the user and the responses of an AI chat program such as Mistral AI.
- HTTP-cache: Contains a simple http cache application that stores the cached content of websites from simple get requests.
  
Both programs are written completely in rust and compiled to WebAssembly. Later they are run using the Wasmtime platform.
Each program contains a host and a guest module. The guest module contains the main logic of the program while host provides access to the operating system and the Wasmtime runtime.
These simple apps are made to demonstrate the capabilities of Wasm run as headless programs in cloud environments.
By testing capabilities such as data serialization using WIT and access to the network and filesystem from a host program, I've shown that code mobility in three forms of strong, semi-strong and weak, is feasible using WebAssembly.
