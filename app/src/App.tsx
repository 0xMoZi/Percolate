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
  useReadContract,
  useSignTypedData,
} from "wagmi";
import PercolateVerifierArtifact from "./abi/PercolateVerifier.json";
import percolateLogo from "./assets/percolate-logo.png";
import "./App.css";

const RISC_ZERO_ROUTER = "0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187";
const IMAGE_ID = "0xcb34c8c6725243346c9306c6c319fe87f21addce825b188c6f9e974e569acd69";
const LIMIT_SWAP_VM_ROUTER = "0xFC5E565420013D413D55e6cAc1a7F46806c122C6";
const WETH_ADDRESS = "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14";
const USDC_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";
const ORDER_BUILDER_HELPER = "0xc48cb15F37a4c98Fb1e0acbECA5FB5Fa5F6029dd";

const ERC20_APPROVE_ABI = [
  {
    type: "function",
    name: "approve",
    inputs: [
      { name: "spender", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    outputs: [{ type: "bool" }],
    stateMutability: "nonpayable",
  },
] as const;

const ORDER_BUILDER_ABI = [
  {
    type: "function",
    name: "buildOrder",
    inputs: [
      { name: "maker", type: "address" },
      { name: "tokenA", type: "address" },
      { name: "tokenB", type: "address" },
      { name: "reserveA", type: "uint256" },
      { name: "reserveB", type: "uint256" },
      { name: "percolateVerifier", type: "address" },
    ],
    outputs: [
      { name: "orderMaker", type: "address" },
      { name: "traits", type: "uint256" },
      { name: "data", type: "bytes" },
    ],
    stateMutability: "pure",
  },
] as const;

const ORDER_DOMAIN = {
  name: "PercolateSwapVMRouter",
  version: "1.0.0",
  chainId: sepolia.id,
  verifyingContract: LIMIT_SWAP_VM_ROUTER as `0x${string}`,
};

const ORDER_TYPES = {
  Order: [
    { name: "maker", type: "address" },
    { name: "traits", type: "uint256" },
    { name: "data", type: "bytes" },
  ],
} as const;

function Card({
  title,
  icon,
  variant,
  children,
}: {
  title: string;
  icon?: string;
  variant?: "maker" | "taker";
  children: React.ReactNode;
}) {
  return (
    <section className={`card ${variant === "taker" ? "card-taker" : ""}`}>
      <h3>
        {icon && <span aria-hidden>{icon}</span>}
        {title}
      </h3>
      <div className="card-body">{children}</div>
    </section>
  );
}

function StepBadge({ done }: { done: boolean }) {
  return <span className={`badge ${done ? "badge-done" : "badge-pending"}`}>{done ? "✓" : "…"}</span>;
}

function NetworkBadge() {
  return (
    <span className="network-badge">
      <span className="network-dot" />
      Sepolia Testnet
    </span>
  );
}

function WalletBar() {
  const { address, isConnected } = useConnection();
  const connectors = useConnectors();
  const connect = useConnect();
  const disconnect = useDisconnect();

  if (isConnected) {
    return (
      <div className="wallet-bar">
        <span className="wallet-address">{address}</span>
        <button className="btn btn-ghost" onClick={() => disconnect.mutate()}>
          Disconnect
        </button>
      </div>
    );
  }

  return (
    <div className="wallet-bar">
      {connectors.map((connector) => (
        <button key={connector.id} className="btn btn-primary" onClick={() => connect.mutate({ connector })}>
          Connect {connector.name}
        </button>
      ))}
    </div>
  );
}

function SignOrderButton({ percolateVerifier }: { percolateVerifier: string }) {
  const { address } = useConnection();
  const signTypedData = useSignTypedData();
  const [signedOrder, setSignedOrder] = useState<any>(null);

  const orderQuery = useReadContract({
    address: ORDER_BUILDER_HELPER,
    abi: ORDER_BUILDER_ABI,
    functionName: "buildOrder",
    args: [
      address as `0x${string}`,
      USDC_ADDRESS,
      WETH_ADDRESS,
      1000000n,
      10000000000000000n,
      percolateVerifier as `0x${string}`,
    ],
    query: { enabled: !!address },
  });

  function handleSign() {
    if (!orderQuery.data) return;
    const [orderMaker, traits, data] = orderQuery.data;
    signTypedData.mutate(
      {
        domain: ORDER_DOMAIN,
        types: ORDER_TYPES,
        primaryType: "Order",
        message: { maker: orderMaker, traits, data },
      },
      {
        onSuccess: (signature) => {
          setSignedOrder({
            order: { maker: orderMaker, traits: traits.toString(), data },
            signature,
          });
        },
      },
    );
  }

  return (
    <div className="step">
      <div className="step-header">
        <StepBadge done={!!signedOrder} />
        <span>Sign order (EIP-712, gas-free)</span>
      </div>
      <button className="btn btn-primary" onClick={handleSign} disabled={!orderQuery.data || signTypedData.isPending}>
        {signTypedData.isPending ? "Signing…" : "Sign Order"}
      </button>
      {signedOrder && (
        <>
          <p className="success-text">Order signed — share this with your taker:</p>
          <pre className="code-block">
            {JSON.stringify(signedOrder, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2)}
          </pre>
        </>
      )}
      {(orderQuery.error || signTypedData.error) && (
        <p className="error-text">{(orderQuery.error || signTypedData.error)?.message}</p>
      )}
    </div>
  );
}

function ApproveTokenButton({ tokenAddress, label }: { tokenAddress: `0x${string}`; label: string }) {
  const write = useWriteContract();

  return (
    <button
      className="btn btn-secondary"
      disabled={write.isPending}
      onClick={() =>
        write.mutate({
          address: tokenAddress,
          abi: ERC20_APPROVE_ABI,
          functionName: "approve",
          args: [LIMIT_SWAP_VM_ROUTER, 2n ** 256n - 1n],
          chainId: sepolia.id,
        })
      }
    >
      {write.isPending ? "Confirming…" : write.isSuccess ? `${label} ✓` : `Approve ${label}`}
    </button>
  );
}

function DeployAndSetRootButton({ root, onDeployed }: { root: string; onDeployed: (addr: string) => void }) {
  const { address } = useConnection();
  const deploy = useDeployContract();
  const write = useWriteContract();
  const [verifierAddress, setVerifierAddress] = useState<string | null>(null);
  const deployReceipt = useWaitForTransactionReceipt({ hash: deploy.data });

  useEffect(() => {
    if (deployReceipt.data?.contractAddress && !write.isPending && !write.isSuccess) {
      setVerifierAddress(deployReceipt.data.contractAddress);
      onDeployed(deployReceipt.data.contractAddress);
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

  const busy = deploy.isPending || write.isPending || deployReceipt.isLoading;

  return (
    <div className="step">
      <div className="step-header">
        <StepBadge done={write.isSuccess} />
        <span>Deploy your own PercolateVerifier + set root</span>
      </div>
      <button className="btn btn-primary" onClick={handleClick} disabled={busy}>
        {deploy.isPending && "Confirming deploy in wallet…"}
        {deployReceipt.isLoading && "Waiting for deploy confirmation…"}
        {write.isPending && "Confirming setRoot in wallet…"}
        {!busy && "Deploy Verifier + Set Root"}
      </button>
      {verifierAddress && <p className="muted">Verifier: {verifierAddress}</p>}
      {write.isSuccess && <p className="success-text">Root set — ready to sign orders.</p>}
      {(deploy.error || write.error) && <p className="error-text">{(deploy.error || write.error)?.message}</p>}
    </div>
  );
}

function MakerMode() {
  const [numTakers, setNumTakers] = useState(3);
  const [allowlist, setAllowlist] = useState<{ root: string; takers: any[] } | null>(null);
  const [verifierAddress, setVerifierAddress] = useState<string | null>(null);

  async function handleGenerate() {
    const result = await invoke("generate_allowlist", { n: numTakers });
    setAllowlist(result as any);
    setVerifierAddress(null);
  }

  return (
    <Card title="Maker" icon="🔑" variant="maker">
      <div className="step">
        <div className="step-header">
          <StepBadge done={!!allowlist} />
          <span>Generate allowlist</span>
        </div>
        <div className="row">
          <input
            className="input input-number"
            type="number"
            min={2}
            value={numTakers}
            onChange={(e) => setNumTakers(Number(e.target.value))}
          />
          <button className="btn btn-primary" onClick={handleGenerate}>
            Generate
          </button>
        </div>
        {allowlist && (
          <details className="details">
            <summary>{allowlist.takers.length} takers generated — view secrets</summary>
            <pre className="code-block">{JSON.stringify(allowlist, null, 2)}</pre>
          </details>
        )}
      </div>

      {allowlist && (
        <>
          <DeployAndSetRootButton root={allowlist.root} onDeployed={setVerifierAddress} />

          <div className="step">
            <div className="step-header">
              <StepBadge done={false} />
              <span>Approve tokens to LimitSwapVMRouter</span>
            </div>
            <div className="row">
              <ApproveTokenButton tokenAddress={WETH_ADDRESS} label="WETH" />
              <ApproveTokenButton tokenAddress={USDC_ADDRESS} label="USDC" />
            </div>
          </div>

          {verifierAddress && <SignOrderButton percolateVerifier={verifierAddress} />}
        </>
      )}
    </Card>
  );
}

function TakerMode() {
  const [secret, setSecret] = useState("");
  const [index, setIndex] = useState(0);

  return (
    <Card title="Taker" icon="🔒" variant="taker">
      <p className="notice">
        <strong>Not implemented in this build.</strong> Proof generation runs natively (RISC Zero STARK) and
        completes in seconds — verified via CLI (<code>percolate-host</code>). Compressing that proof to Groth16
        for cheap on-chain verification requires an x86 Docker environment, which is a current tooling constraint
        of RISC Zero itself, not a Percolate design limitation. The on-chain verification path (
        <code>PercolateGate</code> → <code>PercolateVerifier</code> → <code>RiscZeroVerifierRouter</code>) is fully
        implemented and covered by an integration test against a live Sepolia fork.
      </p>
      <div className="step" style={{ opacity: 0.6 }}>
        <input
          className="input"
          placeholder="0x… (your secret)"
          value={secret}
          onChange={(e) => setSecret(e.target.value)}
          disabled
        />
        <input
          className="input input-number"
          type="number"
          placeholder="index"
          value={index}
          onChange={(e) => setIndex(Number(e.target.value))}
          disabled
        />
        <button className="btn btn-secondary" disabled>
          Generate Proof (requires Docker — see README)
        </button>
      </div>
    </Card>
  );
}

function App() {
  return (
    <main className="app">
      <header className="app-header">
        <div className="brand">
          <img src={percolateLogo} alt="Percolate" className="brand-logo" />
          <h1>Percolate</h1>
        </div>
        <p className="subtitle">Private, RISC Zero-verified allowlist gating for SwapVM</p>
        <NetworkBadge />
        <WalletBar />
      </header>
      <div className="grid">
        <MakerMode />
        <TakerMode />
      </div>
      <footer className="app-footer">Powered by SwapVM — © Degensoft Ltd 2025</footer>
    </main>
  );
}

export default App;
