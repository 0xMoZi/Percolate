import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { useConnection, useConnect, useDisconnect, useConnectors } from "wagmi";
import "./App.css";

function WalletTest() {
  const { address, isConnected } = useConnection();
  const connectors = useConnectors();
  const connect = useConnect();
  const disconnect = useDisconnect();

  if (isConnected) {
    return (
      <div>
        <p>Connected: {address}</p>
        <button onClick={() => disconnect.mutate()}>Disconnect</button>
      </div>
    );
  }

  return (
    <div>
      {connectors.map((connector) => (
        <button key={connector.id} onClick={() => connect.mutate({ connector })}>
          Connect via {connector.name}
        </button>
      ))}
    </div>
  );
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [root, setRoot] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  async function testCompute() {
    const result = await invoke<string>("compute_root", {
      leavesHex: ["0x" + "11".repeat(32), "0x" + "22".repeat(32)],
    });
    setRoot(result);
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank" rel="noreferrer">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank" rel="noreferrer">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank" rel="noreferrer">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      {/* 2. Panggil WalletTest di sini */}
      <WalletTest />

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
        {/* Tambahkan type="button" agar tidak memicu onSubmit form */}
        <button type="button" onClick={testCompute}>Test compute_root</button>
        <p>{root}</p>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
}

export default App;
