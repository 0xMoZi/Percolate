import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { sepolia } from "wagmi/chains";
import {
  useConnection,
  useConnect,
  useDisconnect,
  useConnectors,
  useWriteContract,
  useDeployContract,
  useWaitForTransactionReceipt,
} from "wagmi";
import PercolateVerifierArtifact from "./abi/PercolateVerifier.json";
import "./App.css";

const RISC_ZERO_ROUTER = "0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187";
const IMAGE_ID = "0xcb34c8c6725243346c9306c6c319fe87f21addce825b188c6f9e974e569acd69";

function DeployAndSetRootButton({ root }: { root: string }) {
  const { address } = useConnection();
  const deploy = useDeployContract();
  const write = useWriteContract();
  const [verifierAddress, setVerifierAddress] = useState<string | null>(null);

  const deployReceipt = useWaitForTransactionReceipt({ hash: deploy.data });

  useEffect(() => {
    if (deployReceipt.data?.contractAddress && !write.isPending && !write.isSuccess) {
      setVerifierAddress(deployReceipt.data.contractAddress);
      write.mutate({
        address: deployReceipt.data.contractAddress,
        abi: PercolateVerifierArtifact.abi,
        functionName: "setRoot",
        args: [root as `0x${string}`],
        chainId: sepolia.id,
      });
    }
  }, [deployReceipt.data]);

  function handleClick() {
    if (!address) return;
    deploy.mutate({
      abi: PercolateVerifierArtifact.abi,
      bytecode: PercolateVerifierArtifact.bytecode.object as `0x${string}`,
      args: [address, RISC_ZERO_ROUTER, IMAGE_ID],
      chainId: sepolia.id,
    });
  }

  return (
    <div>
      <button onClick={handleClick} disabled={deploy.isPending || write.isPending || deployReceipt.isLoading}>
        {deploy.isPending && "Confirming deploy in wallet..."}
        {deployReceipt.isLoading && "Waiting for deploy confirmation..."}
        {write.isPending && "Confirming setRoot in wallet..."}
        {!deploy.isPending && !write.isPending && !deployReceipt.isLoading && "Deploy Verifier + Set Root"}
      </button>
      {verifierAddress && <p>Your PercolateVerifier: {verifierAddress}</p>}
      {write.isSuccess && <p>Root set! Ready to sign orders.</p>}
      {(deploy.error || write.error) && (
        <p style={{ color: "red" }}>{(deploy.error || write.error)?.message}</p>
      )}
    </div>
  );
}

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

function MakerMode() {
  const [numTakers, setNumTakers] = useState(3);
  const [allowlist, setAllowlist] = useState<{ root: string; takers: any[] } | null>(null);

  async function handleGenerate() {
    const result = await invoke("generate_allowlist", { n: numTakers });
    setAllowlist(result as any);
  }

  return (
    <div>
      <h3>Maker Mode</h3>
      <input
        type="number"
        min={2}
        value={numTakers}
        onChange={(e) => setNumTakers(Number(e.target.value))}
      />
      <button onClick={handleGenerate}>Generate Allowlist</button>

      {allowlist && (
        <>
          <pre>{JSON.stringify(allowlist, null, 2)}</pre>
          <DeployAndSetRootButton root={allowlist.root} />
        </>
      )}
    </div>
  );
}

function TakerMode() {
  const [secret, setSecret] = useState("");
  const [index, setIndex] = useState(0);

  return (
    <div>
      <h3>Taker Mode</h3>
      <input
        placeholder="0x... (secret kamu)"
        value={secret}
        onChange={(e) => setSecret(e.target.value)}
      />
      <input
        type="number"
        placeholder="index"
        value={index}
        onChange={(e) => setIndex(Number(e.target.value))}
      />
      <button
        onClick={() => alert("Not implemented yet - wait for Docker integration for a real proving (Fase 6)")}
      >
        Generate Proof
      </button>
    </div>
  );
}

function App() {
  return (
    <div>
      <WalletTest />
      <MakerMode />
      <TakerMode />
    </div>
  );
}

export default App;
