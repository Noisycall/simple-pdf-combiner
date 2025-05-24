import { useEffect, useState } from "preact/hooks";
import preactLogo from "./assets/preact.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import {
  BaseDirectory,
  create,
  readDir,
  writeFile,
  writeTextFile,
} from "@tauri-apps/plugin-fs";
// when using `"withGlobalTauri": true`, you may use
// const { exists, BaseDirectory } = window.__TAURI__.fs;

// Check if the `$APPDATA/avatar.png` file exists

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState<File>();
  const [val, setVal] = useState("");
  useEffect(async () => {
    // await mkdir('',{baseDir:BaseDirectory.AppCache});
    await writeTextFile("./wow.txt", "wow", {
      baseDir: BaseDirectory.AppCache,
    });
    setVal(await readDir("", { baseDir: BaseDirectory.AppCache }));
  }, []);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main class="container">
      <h1>Welcome to Tauri + Preact</h1>

      <div class="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://preactjs.com" target="_blank">
          <img src={preactLogo} class="logo preact" alt="Preact logo" />
        </a>
        <div>Val is {JSON.stringify(val)}</div>
      </div>
      <p>Click on the Tauri, Vite, and Preact logos to learn more.</p>
      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        {/*<input*/}
        {/*  id="greet-input"*/}
        {/*  onInput={(e) => setName(e.currentTarget.value)}*/}
        {/*  placeholder="Enter a name..."*/}

        {/*/>*/}

        <input
          type="file"
          onChange={async (evt) => {
            console.log(evt.currentTarget.files![0]);
            let val = evt.currentTarget.files![0];
            let file = await create(val.name, {
              baseDir: BaseDirectory.AppCache,
            });
            await file.close();
            await writeFile(val.name, val.stream(), {
              baseDir: BaseDirectory.AppCache,
            });
          }}
          accept="application/pdf"
        />
        <button type="submit">Greet</button>
      </form>
      <p>{name}</p>
    </main>
  );
}

export default App;
