# Pixardis Web IDE

> Compilers. Pixels. Regret. All running in your browser.

## What the hell is this?

Pixardis was born when a colleague bought a pixel display off AliExpress so cheap it barely sent a signal without crying.  No docs, no spec, and the software looked like malware written by interns. So he wrote a VM spec and handed it off as a compilers assignment.  

I laughed, then rewrote everything in Rust because his JavaScript VM ran like a wounded 386 on a heat stroke.  Then I made it compile and run in the browser because I enjoy pain and hate free time.

Now you’ve got an IDE for a language no one asked for, controlling a display that shouldn’t exist, in a tech stack nobody understands.  
Use it. Don’t. I already won.

---

## 🔥 Try It Live

[**▶ Open Pixardis Web IDE**](https://keithbugeja.github.io/pixardis-web/frontend/index.html)

---

## Features

- ✍️ **Monaco Editor** – Because writing stack code deserves syntax highlighting  
- 🦀 **Rust Compiler + VM** – Fast, safe, and deeply offended by garbage collection  
- 🌐 **WebAssembly** – Makes your browser pretend it’s a real OS  
- 🎨 **Canvas Output** – Because pixels still matter  
- 🧪 **Example Programs** – From `fibonacci` to `snake`, because why not?

---

## Project Layout

```
pixardis-web/
├── compiler/     # Rust compiler for the Pixardis language
├── vm/           # Rust virtual machine (backend)
├── shared/       # Shared logic between compiler and VM
├── web/          # WebAssembly glue
├── frontend/     # Monaco-based web UI
```

---

## Running It Locally

**Requirements:**

- Rust + [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)
- A local web server (`basic-http-server`, `serve`, whatever you trust to not break)

**Steps:**

1. Build the WebAssembly bindings:
   ```bash
   wasm-pack build --target web web/
   ```

2. Serve the frontend:
   ```bash
   cd frontend
   some-web-server .
   ```

3. Open the browser and let the stack abuse begin.

---

## Example Programs

Found in `compiler/examples/`. Load them in the editor, hit compile, and enjoy watching pixels dance like it’s 1994.

- `snake.ps` – You already know  
- `rainbow.ps` – Bright colours, no purpose  
- `fibonacci.ps` – Academic pain  
- `fancy_clock.ps` – Vaguely functional

---

## Why does this exist?

Because:
- That display was an insult to hardware
- JavaScript isn’t a runtime, it’s a regret
- Writing compilers is cheaper than therapy

---

## License

MIT, because the GPL takes too long to read and I have pixels to push.

---

## Final Notes

Made with Rust. Forged in spite.
If it breaks, you get to keep both pieces.
