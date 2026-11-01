import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import {
  Activity,
  ArrowRight,
  ArrowUpRight,
  Check,
  ChevronRight,
  Clock3,
  Compass,
  Flame,
  Grid2X2,
  Heart,
  Info,
  List,
  LockKeyhole,
  Plus,
  Search,
  ShieldCheck,
  Sparkles,
  TrendingUp,
  Users,
  Wallet,
  X,
  Zap,
} from "lucide-react";
import { sampleTokens, type Token } from "./data";
import { AMM_V4_BUILD_CAPABILITY, presentAmmV4 } from "./ammV4Presentation";
import { sampleState } from "./indexerAdapter";
import { localFeaturePlans } from "./localFeaturePresentation";
import {
  getBrowserStorage,
  nextDraftId,
  orderNewFeed,
  readDrafts,
  readWatchlist,
  toggleWatchlist,
  validateCreateV2Metadata,
  validateLocalDraftOptions,
  writeDrafts,
  writeWatchlist,
  type DraftRecord,
} from "./previewState";

type Feed = "trending" | "new" | "graduating" | "watchlist";
type Modal = "create" | "wallet" | "how" | null;

function draftToken(draft: DraftRecord): Token {
  const minutes = Math.max(
    0,
    Math.floor((Date.now() - draft.createdAt) / 60_000),
  );
  const age =
    minutes === 0
      ? "now"
      : minutes < 60
        ? `${minutes}m`
        : minutes < 1440
          ? `${Math.floor(minutes / 60)}h`
          : `${Math.floor(minutes / 1440)}d`;
  return {
    id: draft.id,
    name: draft.name,
    symbol: draft.symbol,
    description: draft.description || "A new idea is taking shape on Beam.",
    icon: "✦",
    colors: ["#4d76f8", "#9d61ed"],
    marketCap: "$0",
    volume: "$0",
    holders: 0,
    age,
    progress: 0,
    change: 0,
    isDraft: true,
  };
}

const chartPoints =
  "0,72 25,68 44,78 65,56 86,62 103,44 123,52 145,34 162,39 180,25 205,36 228,12 252,20";
const previewLifecycle = sampleState(sampleTokens);

function TokenArt({
  token,
  size = "normal",
}: {
  token: Token;
  size?: "normal" | "large";
}) {
  return (
    <div
      className={`token-art ${size === "large" ? "token-art-large" : ""}`}
      style={{
        background: `linear-gradient(145deg, ${token.colors[0]}, ${token.colors[1]})`,
      }}
      aria-hidden="true"
    >
      <span>{token.icon}</span>
      <i className="art-orbit art-orbit-one" />
      <i className="art-orbit art-orbit-two" />
    </div>
  );
}

function MiniChart({ negative = false }: { negative?: boolean }) {
  return (
    <svg
      className={`mini-chart ${negative ? "negative" : ""}`}
      viewBox="0 0 252 90"
      preserveAspectRatio="none"
      role="img"
      aria-label="Illustrative sample chart"
    >
      <defs>
        <linearGradient
          id={negative ? "chart-fill-negative" : "chart-fill"}
          x1="0"
          x2="0"
          y1="0"
          y2="1"
        >
          <stop
            offset="0"
            stopColor={negative ? "#8d67ff" : "#4d85ff"}
            stopOpacity=".24"
          />
          <stop
            offset="1"
            stopColor={negative ? "#8d67ff" : "#4d85ff"}
            stopOpacity="0"
          />
        </linearGradient>
      </defs>
      <polygon
        points={`0,90 ${chartPoints} 252,90`}
        fill={`url(#${negative ? "chart-fill-negative" : "chart-fill"})`}
      />
      <polyline
        points={chartPoints}
        fill="none"
        stroke={negative ? "#9b74ff" : "#5e95ff"}
        strokeWidth="3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function TokenCard({
  token,
  saved,
  onOpen,
  onSave,
  view,
}: {
  token: Token;
  saved: boolean;
  onOpen: () => void;
  onSave: () => void;
  view: "grid" | "list";
}) {
  return (
    <article
      className={`token-card ${view === "list" ? "token-card-list" : ""}`}
    >
      <button
        className="card-open"
        onClick={onOpen}
        aria-label={`View ${token.name}`}
      >
        <div className="card-top">
          <TokenArt token={token} />
          <div className="token-heading">
            <div className="name-line">
              <h3>{token.name}</h3>
              <span className="ticker">${token.symbol}</span>
            </div>
            <p>{token.description}</p>
          </div>
        </div>
        <div className="chart-wrap">
          <MiniChart negative={token.change < 0} />
        </div>
        <div className="card-metrics">
          <div>
            <span>Market cap</span>
            <strong>{token.marketCap}</strong>
          </div>
          <div>
            <span>24h volume</span>
            <strong>{token.volume}</strong>
          </div>
          <div>
            <span>Change</span>
            <strong className={token.change < 0 ? "down" : "up"}>
              {token.change > 0 ? "+" : ""}
              {token.change}%
            </strong>
          </div>
        </div>
        <div className="card-progress">
          <div className="progress-copy">
            <span>Sample curve progress</span>
            <strong>{token.progress}%</strong>
          </div>
          <div className="progress-track">
            <span style={{ width: `${token.progress}%` }} />
          </div>
        </div>
      </button>
      <div className="card-foot">
        <span>
          <Clock3 size={13} />{" "}
          {token.age === "now" ? "just now" : `${token.age} ago`}
        </span>
        <span>
          <Users size={13} /> {token.holders} holders
        </span>
        {token.isDraft && <span className="draft-pill">Local draft</span>}
        <button
          className={`save-button ${saved ? "saved" : ""}`}
          onClick={onSave}
          aria-label={
            saved
              ? `Remove ${token.name} from watchlist`
              : `Add ${token.name} to watchlist`
          }
        >
          <Heart size={17} fill={saved ? "currentColor" : "none"} />
        </button>
      </div>
    </article>
  );
}

function App() {
  const [feed, setFeed] = useState<Feed>("trending");
  const [view, setView] = useState<"grid" | "list">("grid");
  const [search, setSearch] = useState("");
  const [modal, setModal] = useState<Modal>(null);
  const [selected, setSelected] = useState<Token | null>(null);
  const [drafts, setDrafts] = useState<DraftRecord[]>(() =>
    readDrafts(getBrowserStorage()),
  );
  const [saved, setSaved] = useState<string[]>(() =>
    readWatchlist(getBrowserStorage()),
  );
  const [name, setName] = useState("");
  const [symbol, setSymbol] = useState("");
  const [description, setDescription] = useState("");
  const [metadataUri, setMetadataUri] = useState("");
  const [creatorFeeWallet, setCreatorFeeWallet] = useState("");
  const [donationChoice, setDonationChoice] = useState<"none" | "donate.gg">(
    "none",
  );
  const [charityName, setCharityName] = useState("");
  const [callout, setCallout] = useState("");
  const [livestreamUrl, setLivestreamUrl] = useState("");
  const [formError, setFormError] = useState<string | null>(null);
  const [storageWarning, setStorageWarning] = useState(false);
  const usedDraftIds = useRef(new Set(drafts.map((draft) => draft.id)));
  const submittedDraft = useRef(false);
  const dialogPanel = useRef<HTMLDivElement | null>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const dialogWasOpen = useRef(false);
  const dialogOpen = modal !== null || selected !== null;
  const selectedDraft = selected?.isDraft
    ? drafts.find((draft) => draft.id === selected.id)
    : undefined;
  const selectedPlans = selectedDraft ? localFeaturePlans(selectedDraft) : [];
  const selectedLifecycle = selected
    ? presentAmmV4(previewLifecycle, selected.id)
    : null;

  useEffect(() => {
    const storage = getBrowserStorage();
    const watchlistSaved = writeWatchlist(storage, saved);
    const draftsSaved = writeDrafts(storage, drafts);
    setStorageWarning(!watchlistSaved || !draftsSaved);
  }, [saved, drafts]);
  useEffect(() => {
    if (modal === "create") submittedDraft.current = false;
  }, [modal]);
  useEffect(() => {
    if (!dialogOpen) {
      if (dialogWasOpen.current) returnFocus.current?.focus();
      dialogWasOpen.current = false;
      returnFocus.current = null;
      return;
    }
    const panel = dialogPanel.current;
    if (!panel) return;
    if (
      !dialogWasOpen.current &&
      document.activeElement instanceof HTMLElement
    ) {
      returnFocus.current = document.activeElement;
    }
    dialogWasOpen.current = true;
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const focusable = () => [
      ...panel.querySelectorAll<HTMLElement>(
        "button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])",
      ),
    ];
    (focusable()[0] || panel).focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        setModal(null);
        setSelected(null);
      }
      if (event.key !== "Tab") return;
      const items = focusable();
      if (items.length === 0) {
        event.preventDefault();
        panel.focus();
        return;
      }
      const first = items[0];
      const last = items[items.length - 1];
      if (
        event.shiftKey &&
        (document.activeElement === first ||
          !panel.contains(document.activeElement))
      ) {
        event.preventDefault();
        last.focus();
      } else if (
        !event.shiftKey &&
        (document.activeElement === last ||
          !panel.contains(document.activeElement))
      ) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("keydown", onKey);
      document.body.style.overflow = previousOverflow;
    };
  }, [dialogOpen, modal, selected]);

  const tokens = useMemo(() => {
    const draftTokens = drafts.map(draftToken);
    let result = orderNewFeed(draftTokens, sampleTokens);
    if (feed === "watchlist")
      result = result.filter((token) => saved.includes(token.id));
    if (feed === "graduating")
      result = [...result].sort((a, b) => b.progress - a.progress);
    const query = search.trim().toLowerCase();
    if (query)
      result = result.filter((token) =>
        `${token.name} ${token.symbol} ${token.description}`
          .toLowerCase()
          .includes(query),
      );
    return result;
  }, [drafts, feed, saved, search]);

  const toggleSaved = (id: string) =>
    setSaved((current) => toggleWatchlist(current, id));
  const openExplore = () =>
    document.getElementById("explore")?.scrollIntoView({ behavior: "smooth" });

  const makeDraft = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const cleanName = name.trim();
    const cleanSymbol = symbol
      .trim()
      .toUpperCase()
      .replace(/[^A-Z0-9]/g, "");
    const cleanUri = metadataUri.trim();
    const cleanWallet = creatorFeeWallet.trim();
    const cleanCharity = charityName.trim();
    const cleanCallout = callout.trim();
    const cleanStream = livestreamUrl.trim();
    const error =
      validateCreateV2Metadata(cleanName, cleanSymbol, cleanUri) ??
      validateLocalDraftOptions(
        cleanWallet,
        cleanCharity,
        cleanCallout,
        cleanStream,
      );
    if (error) {
      setFormError(error);
      return;
    }
    if (submittedDraft.current) return;
    submittedDraft.current = true;
    const id = nextDraftId(usedDraftIds.current, Date.now());
    usedDraftIds.current.add(id);
    const draft: DraftRecord = {
      id,
      name: cleanName,
      symbol: cleanSymbol,
      description: description.trim(),
      metadataUri: cleanUri,
      createdAt: Date.now(),
      creatorFeeWallet: cleanWallet,
      donationChoice,
      charityName: donationChoice === "donate.gg" ? cleanCharity : "",
      callout: cleanCallout,
      livestreamUrl: cleanStream,
    };
    setDrafts((current) => [draft, ...current].slice(0, 100));
    setFeed("new");
    setSearch("");
    setModal(null);
    setSelected(draftToken(draft));
    setName("");
    setSymbol("");
    setDescription("");
    setMetadataUri("");
    setCreatorFeeWallet("");
    setDonationChoice("none");
    setCharityName("");
    setCallout("");
    setLivestreamUrl("");
    setFormError(null);
  };

  return (
    <div className="site-shell">
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />
      <header className="site-header">
        <div className="header-inner container">
          <button
            className="brand"
            onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
            aria-label="Beam home"
          >
            <span className="brand-icon">
              <img src="/beam-mark.png" alt="" />
            </span>
            <span>
              beam<span className="brand-dot">.</span>
            </span>
          </button>
          <nav className="main-nav" aria-label="Main navigation">
            <button className="nav-active" onClick={openExplore}>
              Explore
            </button>
            <button onClick={() => setModal("how")}>How it works</button>
            <a
              href="https://github.com/shawkkkkk/beam"
              target="_blank"
              rel="noreferrer"
            >
              Docs <ArrowUpRight size={14} />
            </a>
          </nav>
          <div className="header-actions">
            <span className="preview-status">
              <span /> Preview mode
            </span>
            <button
              className="btn btn-quiet wallet-button"
              onClick={() => setModal("wallet")}
            >
              <Wallet size={17} /> Wallet unavailable
            </button>
            <button
              className="btn btn-primary header-create"
              onClick={() => setModal("create")}
            >
              <Plus size={17} /> Create token
            </button>
          </div>
        </div>
      </header>

      <main>
        <div className="preview-banner">
          <Info size={15} />
          <span>
            Interface preview with sample market data. Token creation and
            trading are not live.
            {storageWarning &&
              " Browser storage is unavailable; local drafts and watchlists may disappear on refresh."}
          </span>
        </div>
        <section className="hero container">
          <div className="hero-copy">
            <div className="eyebrow">
              <span className="eyebrow-line" /> THE RAYDIUM-FIRST LAUNCHPAD
            </div>
            <h1>
              Launch bright.
              <br />
              <em>Stay in orbit.</em>
            </h1>
            <p>
              Every great idea starts with a spark. Discover new tokens, build
              your community, and explore the planned path to Raydium AMM v4.
            </p>
            <div className="hero-actions">
              <button
                className="btn btn-primary btn-large"
                onClick={() => setModal("create")}
              >
                Create a token <ArrowUpRight size={18} />
              </button>
              <button
                className="btn btn-secondary btn-large"
                onClick={openExplore}
              >
                Explore launches <ArrowRight size={18} />
              </button>
            </div>
            <div className="hero-proof">
              <span className="proof-icon">
                <ShieldCheck size={15} />
              </span>
              <span>Built for a clearer path to Raydium</span>
              <span className="proof-divider" />
              <span>
                Planned Beam fee: 0.42% community / 0.05% treasury / 0.03%
                founder; creator payout funding pending
              </span>
            </div>
          </div>
          <div className="hero-visual" aria-label="Beam ray gun brand artwork">
            <div className="visual-grid" />
            <div className="visual-ring visual-ring-one" />
            <div className="visual-ring visual-ring-two" />
            <div className="hero-spark spark-one">✦</div>
            <div className="hero-spark spark-two">✧</div>
            <img
              className="hero-logo"
              src="/beam-mark.png"
              alt="Minimal Beam ray gun logo"
            />
            <div className="visual-card">
              <span className="visual-card-label">THE BEAM EFFECT</span>
              <strong>Find your spark.</strong>
              <span>Launch → Curve → Planned AMM v4</span>
            </div>
            <div className="visual-caption">01 / A NEW WAY TO LAUNCH</div>
          </div>
        </section>

        <section
          className="signal-strip container"
          aria-label="Beam launch approach"
        >
          <div>
            <span className="signal-icon blue">
              <Zap size={19} />
            </span>
            <span>
              <strong>Preview a token draft</strong>
              <small>
                {storageWarning
                  ? "Temporary until storage works"
                  : "Saved locally in this browser"}
              </small>
            </span>
          </div>
          <div>
            <span className="signal-icon purple">
              <TrendingUp size={19} />
            </span>
            <span>
              <strong>Grow on the curve</strong>
              <small>Sample curve progress</small>
            </span>
          </div>
          <div>
            <span className="signal-icon violet">
              <Sparkles size={19} />
            </span>
            <span>
              <strong>Plan an AMM v4 migration</strong>
              <small>Selected target; adapter still gated</small>
            </span>
          </div>
        </section>

        <section className="market-section container" id="explore">
          <div className="section-intro">
            <div>
              <div className="eyebrow section-eyebrow">
                <span className="eyebrow-line" /> THE DISCOVERY FEED
              </div>
              <h2>
                Find the next <span>bright idea.</span>
              </h2>
              <p>Explore communities taking shape on Beam.</p>
            </div>
            <div className="market-badge">
              <Activity size={16} /> Sample market view
            </div>
          </div>
          <div className="market-toolbar">
            <div className="feed-tabs" role="tablist" aria-label="Token feed">
              <button
                role="tab"
                aria-selected={feed === "trending"}
                className={feed === "trending" ? "active" : ""}
                onClick={() => setFeed("trending")}
              >
                <Flame size={16} /> Trending
              </button>
              <button
                role="tab"
                aria-selected={feed === "new"}
                className={feed === "new" ? "active" : ""}
                onClick={() => setFeed("new")}
              >
                <Clock3 size={16} /> New
              </button>
              <button
                role="tab"
                aria-selected={feed === "graduating"}
                className={feed === "graduating" ? "active" : ""}
                onClick={() => setFeed("graduating")}
              >
                <Zap size={16} /> Graduating
              </button>
              <button
                role="tab"
                aria-selected={feed === "watchlist"}
                className={feed === "watchlist" ? "active" : ""}
                onClick={() => setFeed("watchlist")}
              >
                <Heart size={16} /> Watchlist
              </button>
            </div>
            <div className="toolbar-right">
              <label className="search-box">
                <Search size={17} />
                <input
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder="Search tokens..."
                  aria-label="Search tokens"
                />
              </label>
              <div className="view-switch">
                <button
                  className={view === "grid" ? "selected" : ""}
                  onClick={() => setView("grid")}
                  aria-label="Grid view"
                >
                  <Grid2X2 size={17} />
                </button>
                <button
                  className={view === "list" ? "selected" : ""}
                  onClick={() => setView("list")}
                  aria-label="List view"
                >
                  <List size={18} />
                </button>
              </div>
            </div>
          </div>
          {tokens.length > 0 ? (
            <div className={`token-grid ${view === "list" ? "list-view" : ""}`}>
              {tokens.map((token) => (
                <TokenCard
                  key={token.id}
                  token={token}
                  saved={saved.includes(token.id)}
                  onOpen={() => setSelected(token)}
                  onSave={() => toggleSaved(token.id)}
                  view={view}
                />
              ))}
            </div>
          ) : (
            <div className="empty-state">
              <Compass size={28} />
              <h3>
                {feed === "watchlist" && !search
                  ? "Your watchlist is empty"
                  : "No tokens found"}
              </h3>
              <p>
                {feed === "watchlist" && !search
                  ? "Tap the heart on a token to save it here."
                  : "Try another name or symbol."}
              </p>
            </div>
          )}
        </section>

        <section className="closing-cta container">
          <div className="closing-glow" />
          <div className="closing-copy">
            <span className="eyebrow">YOUR IDEA IS NEXT</span>
            <h2>
              Ready to make
              <br />
              some noise?
            </h2>
            <p>Every legendary chart started with someone pressing launch.</p>
            <button
              className="btn btn-white"
              onClick={() => setModal("create")}
            >
              Start creating <ArrowUpRight size={18} />
            </button>
          </div>
          <div className="closing-mark">
            <img src="/beam-mark.png" alt="" />
          </div>
        </section>
      </main>

      <footer className="site-footer container">
        <div className="footer-brand">
          <span className="brand-icon">
            <img src="/beam-mark.png" alt="" />
          </span>
          <strong>
            beam<span>.</span>
          </strong>
          <p>Launch bright. Stay in orbit.</p>
        </div>
        <div className="footer-links">
          <button onClick={() => setModal("how")}>How it works</button>
          <a
            href="https://github.com/shawkkkkk/beam"
            target="_blank"
            rel="noreferrer"
          >
            GitHub <ArrowUpRight size={13} />
          </a>
          <span>Built for what comes next.</span>
        </div>
      </footer>

      {(modal || selected) && (
        <div
          className="modal-backdrop"
          onMouseDown={(event) => {
            if (event.target === event.currentTarget) {
              setModal(null);
              setSelected(null);
            }
          }}
        >
          <div
            className="modal-panel"
            ref={dialogPanel}
            tabIndex={-1}
            role="dialog"
            aria-modal="true"
            aria-label={
              selected
                ? `${selected.name} token details`
                : modal === "create"
                  ? "Create a token preview"
                  : modal === "wallet"
                    ? "Wallet connection"
                    : "How Beam works"
            }
          >
            <button
              className="modal-close"
              onClick={() => {
                setModal(null);
                setSelected(null);
              }}
              aria-label="Close dialog"
            >
              <X size={20} />
            </button>
            {selected ? (
              <>
                <div className="detail-heading">
                  <TokenArt token={selected} size="large" />
                  <div>
                    <span className="modal-kicker">
                      {selected.isDraft ? "LOCAL PREVIEW" : "SAMPLE TOKEN"}
                    </span>
                    <h2>{selected.name}</h2>
                    <p>${selected.symbol}</p>
                  </div>
                </div>
                <p className="detail-description">{selected.description}</p>
                {selectedLifecycle && (
                  <section
                    className="amm-v4-status"
                    role="region"
                    aria-labelledby="amm-v4-status-heading"
                  >
                    <h3 id="amm-v4-status-heading">Curve and AMM v4 status</h3>
                    <p>
                      {selected.isDraft
                        ? "Local draft — no token or bonding curve has been created."
                        : selectedLifecycle.label}
                    </p>
                    <p>
                      Migration in this build:{" "}
                      {AMM_V4_BUILD_CAPABILITY.migration}. Verified AMM v4
                      liquidity: unavailable (no authenticated source).
                    </p>
                  </section>
                )}
                {selectedPlans.length > 0 && (
                  <section
                    className="draft-options"
                    aria-label="Local feature plans"
                  >
                    <strong>Local plans only — all actions unavailable</strong>
                    {selectedPlans.map((plan) => (
                      <p key={plan.kind}>
                        {plan.label}: {plan.value}. {plan.reason}
                      </p>
                    ))}
                  </section>
                )}
                <div className="detail-chart">
                  <MiniChart negative={selected.change < 0} />
                </div>
                <div className="detail-stats">
                  <div>
                    <span>Market cap</span>
                    <strong>{selected.marketCap}</strong>
                  </div>
                  <div>
                    <span>Holders</span>
                    <strong>{selected.holders}</strong>
                  </div>
                  <div>
                    <span>Curve progress</span>
                    <strong>{selected.progress}%</strong>
                  </div>
                </div>
                <div className="detail-progress">
                  <span style={{ width: `${selected.progress}%` }} />
                </div>
                <div className="unavailable-note">
                  <LockKeyhole size={18} />
                  <span>
                    {selected.isDraft
                      ? "This is a local draft in this browser, with no token minted."
                      : "This is sample market data."}{" "}
                    Trading and migration are unavailable in this preview.
                  </span>
                </div>
                <button className="btn btn-primary modal-main-button" disabled>
                  Trading not live yet
                </button>
              </>
            ) : null}
            {modal === "create" && !selected ? (
              <>
                <span className="modal-kicker">START SOMETHING BRIGHT</span>
                <h2>Create a token</h2>
                <p className="modal-subtitle">
                  Shape your idea. This draft stays in this browser; it does not
                  upload metadata or mint a token.
                </p>
                <form className="create-form" onSubmit={makeDraft}>
                  <label>
                    Token name
                    <input
                      value={name}
                      onChange={(event) => setName(event.target.value)}
                      minLength={2}
                      maxLength={32}
                      required
                      placeholder="e.g. Cosmic Cat"
                    />
                  </label>
                  <label>
                    Ticker symbol
                    <input
                      value={symbol}
                      onChange={(event) =>
                        setSymbol(
                          event.target.value
                            .toUpperCase()
                            .replace(/[^A-Z0-9]/g, ""),
                        )
                      }
                      minLength={2}
                      maxLength={13}
                      required
                      placeholder="e.g. COSMIC"
                    />
                  </label>
                  <label>
                    Metadata URI (optional for this local draft)
                    <input
                      value={metadataUri}
                      onChange={(event) => {
                        setMetadataUri(event.target.value);
                        setFormError(null);
                      }}
                      maxLength={200}
                      placeholder="Existing metadata URI; no upload occurs"
                    />
                  </label>
                  <label>
                    Description
                    <textarea
                      value={description}
                      onChange={(event) => setDescription(event.target.value)}
                      maxLength={180}
                      rows={3}
                      placeholder="Tell everyone what makes it special"
                    />
                  </label>
                  <p className="metadata-hint">
                    Optional product plans below are saved locally. They do not
                    set a creator fee, register a charity, publish a callout, or
                    start a stream.
                  </p>
                  <label>
                    Creator fee wallet address (local plan)
                    <input
                      value={creatorFeeWallet}
                      onChange={(event) =>
                        setCreatorFeeWallet(event.target.value)
                      }
                      maxLength={44}
                      placeholder="Optional Solana address"
                    />
                  </label>
                  <label>
                    Charity/donation choice
                    <select
                      value={donationChoice}
                      onChange={(event) =>
                        setDonationChoice(
                          event.target.value as "none" | "donate.gg",
                        )
                      }
                    >
                      <option value="none">No donation plan</option>
                      <option value="donate.gg">
                        Interested in donate.gg (not connected)
                      </option>
                    </select>
                  </label>
                  {donationChoice === "donate.gg" && (
                    <label>
                      Charity note (unverified)
                      <input
                        value={charityName}
                        onChange={(event) => setCharityName(event.target.value)}
                        maxLength={80}
                        placeholder="Name of intended charity"
                      />
                    </label>
                  )}
                  <label>
                    Callout text (local plan)
                    <textarea
                      value={callout}
                      onChange={(event) => setCallout(event.target.value)}
                      maxLength={180}
                      rows={2}
                      placeholder="Optional community callout"
                    />
                  </label>
                  <label>
                    Livestream URL (local plan)
                    <input
                      type="url"
                      value={livestreamUrl}
                      onChange={(event) => setLivestreamUrl(event.target.value)}
                      maxLength={300}
                      placeholder="https://…"
                    />
                  </label>
                  <p className="metadata-hint">
                    CreateV2 limits: name 32, symbol 13, metadata URI 200 UTF-8
                    bytes. This form validates those byte limits locally.
                  </p>
                  {formError && (
                    <p className="form-error" role="alert">
                      {formError}
                    </p>
                  )}
                  <div className="unavailable-note">
                    <Info size={17} />
                    <span>
                      This button saves a local draft only. It does not connect
                      a wallet, upload files, or call the Beam program.
                    </span>
                  </div>
                  <button
                    className="btn btn-primary modal-main-button"
                    type="submit"
                  >
                    Create local preview <ArrowRight size={17} />
                  </button>
                </form>
              </>
            ) : null}
            {modal === "wallet" && !selected ? (
              <>
                <span className="modal-kicker">
                  NOT AVAILABLE IN THIS PREVIEW
                </span>
                <div className="modal-icon">
                  <Wallet size={28} />
                </div>
                <h2>No wallet connection here.</h2>
                <p className="modal-subtitle">
                  This sample-data interface cannot connect, sign, or submit a
                  transaction. Raydium AMM v4 is the selected migration target;
                  the adapter still needs integration and runtime review.
                </p>
                <button
                  className="btn btn-secondary modal-main-button"
                  onClick={() => setModal(null)}
                >
                  Got it <Check size={17} />
                </button>
              </>
            ) : null}
            {modal === "how" && !selected ? (
              <>
                <span className="modal-kicker">THE BEAM PATH</span>
                <h2>From spark to orbit.</h2>
                <p className="modal-subtitle">
                  Beam is being built around a curve launch and a separate
                  Raydium AMM v4 migration. That adapter is not active here.
                </p>
                <div className="how-steps">
                  <div>
                    <span>01</span>
                    <div>
                      <h3>Launch</h3>
                      <p>
                        Create a token and start a community on a bonding curve.
                      </p>
                    </div>
                    <ChevronRight size={18} />
                  </div>
                  <div>
                    <span>02</span>
                    <div>
                      <h3>Grow</h3>
                      <p>
                        The specified Beam allocation is 0.50%: 0.42% for the
                        community bucket, 0.05% treasury, and 0.03% founder.
                        Creator payout funding is undecided. Collection and
                        distribution are not active in this preview.
                      </p>
                    </div>
                    <ChevronRight size={18} />
                  </div>
                  <div>
                    <span>03</span>
                    <div>
                      <h3>Graduate</h3>
                      <p>
                        Curve completion and migration are distinct states.
                        Raydium AMM v4 is selected, but its adapter is not
                        integrated here and this preview cannot create a pool.
                      </p>
                    </div>
                    <ChevronRight size={18} />
                  </div>
                </div>
                <div className="unavailable-note">
                  <Info size={17} />
                  <span>
                    These are product goals. The current Beam code is not
                    deployed.
                  </span>
                </div>
              </>
            ) : null}
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
