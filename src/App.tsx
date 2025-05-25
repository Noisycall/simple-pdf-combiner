import { useState } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { BaseDirectory, create, writeFile } from "@tauri-apps/plugin-fs";
import { save } from "@tauri-apps/plugin-dialog";
import { downloadDir } from "@tauri-apps/api/path";
// when using `"withGlobalTauri": true`, you may use
// const { exists, BaseDirectory } = window.__TAURI__.fs;

// Check if the `$APPDATA/avatar.png` file exists

function App() {
  addEventListener("dragenter", (event) => {
    event.stopPropagation();
    event.preventDefault();
  });
  addEventListener("dragover", (event) => {
    event.stopPropagation();
    event.preventDefault();
  });
  addEventListener("drop", (event) => {
    event.stopPropagation();
    event.preventDefault();
  });
  const [file1, setFile1] = useState<File>();
  const [file2, setFile2] = useState<File>();

  return (
    <main class="container">
      <form class="row">
        <div
          id={"dragbox1"}
          onDragEnter={(evt) => {
            evt.preventDefault();
          }}
          onDragOver={(evt) => {
            evt.preventDefault();
          }}
          onDrop={async (evt) => {
            console.log("dropped");
            const dt = evt.dataTransfer;
            console.log(dt?.files[0]);
            setFile1(dt?.files[0]);
            let val = dt!.files[0];
            let file = await create(val.name, {
              baseDir: BaseDirectory.AppCache,
            });
            await file.close();
            let blobed = new Uint8Array(await val.arrayBuffer());
            await writeFile(val.name, blobed, {
              baseDir: BaseDirectory.AppCache,
            });
          }}
          style={{
            width: "100px",
            height: "100px",
            background: "red",
            margin: "10px",
            border: "dashed black 5px",
          }}
        >
          {file1?.name}
        </div>
        <div
          id={"dragbox2"}
          onDragEnter={(evt) => {
            evt.stopPropagation();
            evt.preventDefault();
          }}
          onDragExit={(evt) => {
            evt.stopPropagation();
            evt.preventDefault();
          }}
          onDrop={async (evt) => {
            evt.stopPropagation();
            evt.preventDefault();
            const dt = evt.dataTransfer;
            setFile2(dt?.files[0]);
            let val = dt!.files[0];
            let file = await create(val.name, {
              baseDir: BaseDirectory.AppCache,
            });
            await file.close();
            let blobed = new Uint8Array(await val.arrayBuffer());
            await writeFile(val.name, blobed, {
              baseDir: BaseDirectory.AppCache,
            });
          }}
          style={{
            width: "100px",
            height: "100px",
            background: "red",
            margin: "10px",
            border: "dashed black 5px",
          }}
        >
          {file2?.name}
        </div>

        <button
          onClick={async (evt) => {
            evt.preventDefault();
            let result: ArrayBuffer = await invoke("combine_pdf", {
              file1: file1?.name,
              file2: file2?.name,
            });
            console.info("result", result);

            let filePath = await save({
              defaultPath: (await downloadDir()) + "/merged.pdf",
            });
            if (filePath) {
              await writeFile(filePath, new Uint8Array(result));
            }
          }}
        >
          Merge
        </button>
      </form>
    </main>
  );
}

export default App;
