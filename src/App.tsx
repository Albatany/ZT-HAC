import { useEffect, useState, type ReactNode } from "react";
import { api, type GuardStatus, type PortRule, type ProcInfo, type Restriction, type VaultReport } from "./api/security";

type Kind = "ok" | "err" | "warn";
type Toast = { id: number; kind: Kind; msg: string };
type Log = { id: number; time: string; msg: string };
type Ctx = {
  act: <T,>(fn: () => Promise<T>, ok?: string) => Promise<T | undefined>;
  rules: Restriction[]; setRules: (r: Restriction[]) => void;
  guard: GuardStatus; setGuard: (g: GuardStatus) => void;
  ports: PortRule[]; setPorts: (p: PortRule[]) => void;
};

const inp = "w-full rounded-lg bg-bg border border-line px-3 py-2 text-sm text-ink placeholder:text-mute outline-none transition duration-200 focus:border-accent focus:ring-2 focus:ring-accent/20";
const btn = (v: "p" | "g" | "d" = "p") =>
  `rounded-lg px-4 py-2 text-sm font-medium transition duration-200 active:scale-95 disabled:opacity-40 ${
    v === "p" ? "bg-accent text-bg hover:brightness-110" : v === "d" ? "bg-danger/15 text-danger hover:bg-danger/25" : "border border-line text-ink hover:bg-raised hover:border-mute"}`;
const TABS = [
  { id: "overview", label: "1. Overview", icon: "⌁" },
  { id: "apps", label: "2. App restrictor", icon: "⌁" },
  { id: "vault", label: "3. Folder vault", icon: "⌁" },
  { id: "guard", label: "4. Environment guard", icon: "⌁" },
  { id: "net", label: "5. Port stopper", icon: "⌁" },
] as const;
type TabId = (typeof TABS)[number]["id"];

function Card({ title, hint, children }: { title: string; hint?: string; children: ReactNode }) {
  return (
    <section className="rounded-xl border border-line bg-surface p-5 transition duration-200 hover:border-accent/40 hover:-translate-y-0.5">
      <h2 className="text-base font-semibold">{title}</h2>
      {hint && <p className="mt-1 mb-4 text-sm text-mute">{hint}</p>}
      {!hint && <div className="mb-4" />}
      {children}
    </section>
  );
}

function Row({ children }: { children: ReactNode }) {
  return <li className="flex items-center justify-between gap-3 rounded-lg border border-line bg-bg px-3 py-2 text-sm animate-pop transition duration-200 hover:bg-raised">{children}</li>;
}

export default function App() {
  const [tab, setTab] = useState<TabId>("overview");
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [logs, setLogs] = useState<Log[]>([]);
  const [rules, setRules] = useState<Restriction[]>([]);
  const [guard, setGuard] = useState<GuardStatus>({ armed: false, paths: [] });
  const [ports, setPorts] = useState<PortRule[]>([]);

  const log = (msg: string) =>
    setLogs((l) => [{ id: Date.now() + Math.random(), time: new Date().toLocaleTimeString(), msg }, ...l].slice(0, 40));
  const toast = (kind: Kind, msg: string) => {
    const id = Date.now() + Math.random();
    setToasts((t) => [...t, { id, kind, msg }]);
    setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), 3800);
  };
  async function act<T>(fn: () => Promise<T>, ok?: string): Promise<T | undefined> {
    try {
      const r = await fn();
      if (ok) { toast("ok", ok); log(ok); }
      return r;
    } catch (e) {
      toast("err", String(e)); log(String(e));
    }
  }

  useEffect(() => {
    api.listRestrictions().then(setRules).catch(() => {});
    api.guardStatus().then(setGuard).catch(() => {});
    api.listPortRules().then(setPorts).catch(() => {});
    const subs = [
      api.on("app-blocked", (n) => { toast("warn", `Blocked ${n}`); log(`Blocked launch of ${n}`); }),
      api.on("guard-event", (m) => log(m)),
      api.on("guard-locked", (m) => { toast("ok", m); log(m); }),
    ];
    return () => { subs.forEach((s) => s.then((un) => un()).catch(() => {})); };
  }, []);

  const ctx: Ctx = { act, rules, setRules, guard, setGuard, ports, setPorts };

  return (
    <div className="flex h-full flex-col">
      <div className="flex min-h-0 flex-1">
        <aside className="w-60 shrink-0 border-r border-line bg-surface p-4">
          <div className="mb-8 flex items-center gap-3 px-2">
            <span className="h-2.5 w-2.5 rounded-full bg-accent animate-beat" />
            <div>
              <div className="font-semibold leading-tight">ZT-HAC</div>
              <div className="text-xs text-mute">Zero Trust - Host Access Controller, Ready to secure your environment</div>
            </div>
          </div>
          <nav className="space-y-1">
            {TABS.map((t) => (
              <button key={t.id} onClick={() => setTab(t.id)}
                className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition duration-200 active:scale-95 ${
                  tab === t.id ? "bg-accent/15 text-accent" : "text-mute hover:bg-raised hover:text-ink hover:translate-x-1"}`}>
                <span className="w-4 text-center">{t.icon}</span>{t.label}
              </button>
            ))}
          </nav>
        </aside>
        <main className="min-w-0 flex-1 overflow-auto p-8">
          <div key={tab} className="mx-auto max-w-3xl animate-rise space-y-5">
            {tab === "overview" && <Overview c={ctx} logs={logs} />}
            {tab === "apps" && <Apps c={ctx} />}
            {tab === "vault" && <Vault c={ctx} />}
            {tab === "guard" && <Guard c={ctx} />}
            {tab === "net" && <Net c={ctx} />}
          </div>
        </main>
      </div>
      <footer className="border-t border-line bg-surface py-2.5 text-center text-xs text-mute">© Albatany 2026</footer>
      <div className="pointer-events-none fixed right-5 top-5 z-50 space-y-2">
        {toasts.map((t) => (
          <div key={t.id} className={`animate-toast rounded-lg border bg-raised px-4 py-3 text-sm shadow-lg ${
            t.kind === "ok" ? "border-accent/50" : t.kind === "warn" ? "border-warn/50 text-warn" : "border-danger/50 text-danger"}`}>{t.msg}</div>
        ))}
      </div>
    </div>
  );
}

function Stat({ label, value, on }: { label: string; value: string; on?: boolean }) {
  return (
    <div className="rounded-xl border border-line bg-surface p-5 transition duration-200 hover:border-accent/40 hover:-translate-y-0.5">
      <div className="flex items-center gap-2 text-sm text-mute">
        <span className={`h-2 w-2 rounded-full ${on ? "bg-accent animate-beat" : "bg-line"}`} />{label}
      </div>
      <div className="mt-2 text-3xl font-semibold">{value}</div>
    </div>
  );
}

function Overview({ c, logs }: { c: Ctx; logs: Log[] }) {
  return (
    <>
      <h1 className="text-2xl font-semibold">Security overview</h1>
      <div className="grid grid-cols-3 gap-4">
        <Stat label="Restricted apps" value={String(c.rules.filter((r) => !r.unlocked).length)} on={c.rules.some((r) => !r.unlocked)} />
        <Stat label="Environment guard" value={c.guard.armed ? "Armed" : "Off"} on={c.guard.armed} />
        <Stat label="Blocked ports" value={String(c.ports.length)} on={c.ports.length > 0} />
      </div>
      <Card title="Activity" hint="Events from the backend appear here as they happen.">
        <ul className="max-h-72 space-y-2 overflow-auto font-mono text-xs">
          {logs.length === 0 && <li className="text-mute">Nothing yet. Restrict an app or arm the guard to get started.</li>}
          {logs.map((l) => (
            <li key={l.id} className="animate-pop rounded-lg border border-line bg-bg px-3 py-2"><span className="mr-3 text-mute">{l.time}</span>{l.msg}</li>
          ))}
        </ul>
      </Card>
    </>
  );
}

function Apps({ c }: { c: Ctx }) {
  const [name, setName] = useState("");
  const [procs, setProcs] = useState<ProcInfo[]>([]);
  const mut = async (fn: () => Promise<void>, ok: string) => { await c.act(fn, ok); c.setRules(await api.listRestrictions()); };
  return (
    <>
      <h1 className="text-2xl font-semibold">App restrictor</h1>
      <Card title="Restrict an application" hint="Matching processes are terminated immediately until you unlock them. On Windows include .exe.">
        <div className="flex gap-2">
          <input className={inp} placeholder="e.g. firefox" value={name} onChange={(e) => setName(e.target.value)} />
          <button className={btn()} onClick={async () => { if (name.trim()) { await mut(() => api.restrictApp(name), `Restricted ${name}`); setName(""); } }}>Restrict</button>
        </div>
        <button className={`${btn("g")} mt-3`} onClick={async () => setProcs((await c.act(api.listProcesses)) ?? [])}>Scan running processes</button>
        {procs.length > 0 && (
          <ul className="mt-3 max-h-44 space-y-1 overflow-auto">
            {procs.map((p) => (
              <li key={p.pid}><button onClick={() => setName(p.name)} className="flex w-full justify-between rounded-lg px-3 py-1.5 text-left text-sm text-mute transition duration-200 hover:bg-raised hover:text-ink active:scale-[.99]">
                <span>{p.name}</span><span className="font-mono text-xs">{(p.memory_kb / 1024).toFixed(0)} MB</span></button></li>
            ))}
          </ul>
        )}
      </Card>
      <Card title="Active rules">
        <ul className="space-y-2">
          {c.rules.length === 0 && <li className="text-sm text-mute">No restrictions yet.</li>}
          {c.rules.map((r) => (
            <Row key={r.name}>
              <span className="flex items-center gap-2"><span className={`h-2 w-2 rounded-full ${r.unlocked ? "bg-warn" : "bg-danger animate-beat"}`} />{r.name}</span>
              <span className="flex gap-2">
                {r.unlocked
                  ? <button className={btn("g")} onClick={() => mut(() => api.relockApp(r.name), `Locked ${r.name}`)}>Lock</button>
                  : <button className={btn("g")} onClick={() => mut(() => api.unlockApp(r.name), `Unlocked ${r.name}`)}>Unlock</button>}
                <button className={btn("d")} onClick={() => mut(() => api.removeRestriction(r.name), `Removed ${r.name}`)}>Remove</button>
              </span>
            </Row>
          ))}
        </ul>
      </Card>
    </>
  );
}

function Vault({ c }: { c: Ctx }) {
  const [path, setPath] = useState("");
  const [pw, setPw] = useState("");
  const [rep, setRep] = useState<VaultReport | null>(null);
  const [busy, setBusy] = useState(false);
  const go = async (enc: boolean) => {
    setBusy(true);
    const r = await c.act(() => (enc ? api.encryptVault(path, pw) : api.decryptVault(path, pw)), enc ? "Vault sealed" : "Vault opened");
    if (r) setRep(r);
    setBusy(false);
  };
  return (
    <>
      <h1 className="text-2xl font-semibold">Folder vault</h1>
      <Card title="Seal or open a folder" hint="AES-256-GCM with an Argon2id key. Sealed files use the .ztv extension. If you forget the password, the data cannot be recovered.">
        <div className="space-y-3">
          <input className={inp} placeholder="Absolute path to a folder or file" value={path} onChange={(e) => setPath(e.target.value)} />
          <input className={inp} type="password" placeholder="Password (8+ characters)" value={pw} onChange={(e) => setPw(e.target.value)} />
          <div className="flex gap-2">
            <button className={btn()} disabled={busy || !path || !pw} onClick={() => go(true)}>{busy ? "Working…" : "Seal vault"}</button>
            <button className={btn("g")} disabled={busy || !path || !pw} onClick={() => go(false)}>Open vault</button>
          </div>
        </div>
      </Card>
      {rep && (
        <Card title="Result">
          <p className="text-sm">{rep.processed} processed, {rep.skipped} skipped, {rep.errors.length} failed.</p>
          <ul className="mt-2 space-y-1 font-mono text-xs text-danger">{rep.errors.map((e) => <li key={e}>{e}</li>)}</ul>
        </Card>
      )}
    </>
  );
}

function Guard({ c }: { c: Ctx }) {
  const [paths, setPaths] = useState("");
  const [pw, setPw] = useState("");
  return (
    <>
      <h1 className="text-2xl font-semibold">Environment guard</h1>
      <Card title="Protect sensitive files" hint="Watches the files below and seals them when every editor or IDE has closed. The password stays in memory only while the guard is armed.">
        <div className="space-y-3">
          <textarea className={`${inp} h-28 font-mono`} placeholder={"One absolute path per line\n/home/me/app/.env\n/home/me/.ssh/id_rsa"} value={paths} onChange={(e) => setPaths(e.target.value)} />
          <input className={inp} type="password" placeholder="Password (8+ characters)" value={pw} onChange={(e) => setPw(e.target.value)} />
          <div className="flex items-center gap-3">
            <button className={btn()} onClick={async () => { const g = await c.act(() => api.guardArm(paths.split("\n").map((s) => s.trim()).filter(Boolean), pw), "Guard armed"); if (g) c.setGuard(g); }}>Arm guard</button>
            <button className={btn("g")} disabled={!c.guard.armed} onClick={async () => { const g = await c.act(api.guardDisarm, "Guard disarmed"); if (g) c.setGuard(g); }}>Disarm</button>
            <span className="flex items-center gap-2 text-sm text-mute"><span className={`h-2 w-2 rounded-full ${c.guard.armed ? "bg-accent animate-beat" : "bg-line"}`} />{c.guard.armed ? `Guarding ${c.guard.paths.length} file(s)` : "Not armed"}</span>
          </div>
        </div>
      </Card>
    </>
  );
}

function Net({ c }: { c: Ctx }) {
  const [port, setPort] = useState("");
  const [proto, setProto] = useState("tcp");
  const [busy, setBusy] = useState(false);
  return (
    <>
      <h1 className="text-2xl font-semibold">Port stopper</h1>
      <Card title="Block a port" hint="Linux uses iptables through pkexec; Windows uses Windows Firewall and needs Administrator rights.">
        <div className="flex gap-2">
          <input className={inp} inputMode="numeric" placeholder="Port, e.g. 3000" value={port} onChange={(e) => setPort(e.target.value.replace(/\D/g, ""))} />
          <select className={`${inp} w-28`} value={proto} onChange={(e) => setProto(e.target.value)}><option>tcp</option><option>udp</option></select>
          <button className={btn()} disabled={busy || !port} onClick={async () => { setBusy(true); const r = await c.act(() => api.blockPort(Number(port), proto), `Blocked ${proto}/${port}`); if (r) { c.setPorts(r); setPort(""); } setBusy(false); }}>Block</button>
        </div>
      </Card>
      <Card title="Blocked ports">
        <ul className="space-y-2">
          {c.ports.length === 0 && <li className="text-sm text-mute">No ports blocked.</li>}
          {c.ports.map((p) => (
            <Row key={`${p.protocol}${p.port}`}>
              <span className="font-mono">{p.protocol}/{p.port}</span>
              <button className={btn("d")} onClick={async () => { const r = await c.act(() => api.unblockPort(p.port, p.protocol), `Unblocked ${p.protocol}/${p.port}`); if (r) c.setPorts(r); }}>Unblock</button>
            </Row>
          ))}
        </ul>
      </Card>
    </>
  );
}
