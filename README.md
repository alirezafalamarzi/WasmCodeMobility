# Code for my research on code mobility using WebAssembly
## Structure:
### AI-history: Contains a cache application that captures the history of the prompts made by the user and the responses of an AI chat program such as Mistral AI.
<img width="1051" height="591" alt="image" src="https://github.com/user-attachments/assets/7b32dd1d-b781-4487-845f-8d9ba8268a13" />


### HTTP-cache: Contains a simple http cache application that stores the cached content of websites from simple get requests.
<img width="1051" height="591" alt="image" src="https://github.com/user-attachments/assets/7b6eb138-c095-46ee-afee-621613003a30" />


Both programs are written completely in rust and compiled to WebAssembly. Later they are run using the Wasmtime platform.
Each program contains a host and a guest module. The guest module contains the main logic of the program while host provides access to the operating system and the Wasmtime runtime.
These simple apps are made to demonstrate the capabilities of Wasm run as headless programs in cloud environments.
By testing capabilities such as data serialization using WIT and access to the network and filesystem from a host program, I've shown that code mobility in three forms of strong, semi-strong and weak, is feasible using WebAssembly.
