// Analytics consent (GDPR / ePrivacy): Google Analytics loads only after the visitor clicks
// "Accept". Until then nothing is sent to Google and no analytics cookie is set. Declining is
// one click, like accepting, and "Cookie settings" in the footer changes the choice at any time.
// A Global Privacy Control signal counts as declining, so those visitors never see the banner;
// only an Accept given while the signal was already on overrides it.
//
// The measurement id comes from <meta name="fps-analytics"> (site.config.json, analytics).

const id = document.querySelector('meta[name="fps-analytics"]')?.content ?? "";
const KEY = "fps-consent";
const VERSION = 1; // Raise when what we ask consent for changes, so everyone is asked again.
const MAX_AGE_MS = 395 * 24 * 60 * 60 * 1000; // Ask again after 13 months.
const root = document.documentElement;
const gpc = () => navigator.globalPrivacyControl === true;

/** The stored choice: true (accepted), false (declined) or null (none, expired or outdated). */
function stored() {
  try {
    const c = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (!c || c.v !== VERSION || typeof c.analytics !== "boolean") return null;
    if (!(Date.now() - Date.parse(c.at) < MAX_AGE_MS)) return null;
    // An Accept from before the browser started sending GPC gives way to the signal.
    if (c.analytics && gpc() && c.gpc !== true) return false;
    return c.analytics;
  } catch {
    return null;
  }
}

function remember(analytics) {
  try {
    localStorage.setItem(KEY, JSON.stringify({ v: VERSION, analytics, gpc: gpc(), at: new Date().toISOString() }));
  } catch {
    // Private mode: the choice lasts for this page only.
  }
}

let started = false;
function startAnalytics() {
  window[`ga-disable-${id}`] = false;
  if (started) {
    window.gtag("consent", "update", { analytics_storage: "granted" });
    return;
  }
  started = true;
  window.dataLayer = window.dataLayer || [];
  // gtag must queue the arguments object itself, not an array copy.
  window.gtag = function gtag() {
    window.dataLayer.push(arguments);
  };
  gtag("consent", "default", { analytics_storage: "granted", ad_storage: "denied", ad_user_data: "denied", ad_personalization: "denied" });
  gtag("js", new Date());
  gtag("config", id, {
    cookie_domain: location.hostname, // This site only, not every *.mecorelabs.com site.
    cookie_expires: 395 * 24 * 60 * 60, // 13 months from consent: not renewed on later visits.
    cookie_update: false,
    allow_google_signals: false,
    allow_ad_personalization_signals: false,
  });
  const s = document.createElement("script");
  s.async = true;
  s.src = `https://www.googletagmanager.com/gtag/js?id=${encodeURIComponent(id)}`;
  document.head.append(s);
}

/** Removes the cookies Google Analytics set (_ga, _ga_<id>), on this host and its parents. */
function deleteCookies() {
  const parts = location.hostname.split(".");
  const domains = [""];
  for (let i = 0; i < parts.length - 1; i++) domains.push(`; domain=.${parts.slice(i).join(".")}`);
  for (const c of document.cookie.split(";")) {
    const name = c.split("=")[0].trim();
    if (name !== "_ga" && !name.startsWith("_ga_") && name !== "_gid") continue;
    for (const d of domains) document.cookie = `${name}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/${d}`;
  }
}

function stopAnalytics() {
  window[`ga-disable-${id}`] = true;
  if (started) window.gtag("consent", "update", { analytics_storage: "denied" });
  deleteCookies();
}

let banner = null;
let opener = null; // The "Cookie settings" button that reopened the banner gets focus back.
let resize = null;
function hideBanner() {
  if (!banner) return;
  // Move focus out before the banner goes, so keyboard users don't land on <body>.
  if (opener) opener.focus();
  else if (banner.contains(document.activeElement)) {
    const main = document.getElementById("main");
    main?.setAttribute("tabindex", "-1");
    main?.focus({ preventScroll: true });
  }
  banner.remove();
  banner = null;
  opener = null;
  resize?.disconnect();
  root.style.removeProperty("--consent-h");
}

function choose(analytics) {
  remember(analytics);
  hideBanner();
  if (analytics) startAnalytics();
  else stopAnalytics();
}

function showBanner() {
  if (banner) return;
  const choice = stored();
  const current =
    choice !== null
      ? ` You have ${choice ? "accepted" : "declined"} analytics.`
      : gpc()
        ? " Analytics is off because your browser sends a Global Privacy Control signal."
        : "";
  banner = document.createElement("section");
  banner.className = "consent";
  banner.setAttribute("aria-label", "Cookie consent");
  banner.innerHTML = `<div class="container consent-inner">
  <p>May we use <strong>Google Analytics</strong> to count visits and see which pages help? It sets cookies and sends usage data to Google. Nothing is collected unless you accept, and your PDFs never leave your device either way.${current} <a href="/privacy/">Privacy notice</a></p>
  <div class="consent-actions">
    <button class="btn" type="button" data-consent="decline">Decline</button>
    <button class="btn" type="button" data-consent="accept">Accept</button>
  </div>
</div>`;
  banner.addEventListener("click", (e) => {
    const b = e.target instanceof Element ? e.target.closest("button[data-consent]") : null;
    if (b) choose(b.dataset.consent === "accept");
  });
  // Early in the reading order (screen readers meet it first); CSS still pins it to the bottom.
  const skip = document.querySelector("a.skip");
  if (skip) skip.after(banner);
  else document.body.prepend(banner);
  // Keep the end of the page (and the demo's bottom bar) scrollable out from under the banner.
  const pad = () => banner && root.style.setProperty("--consent-h", `${banner.offsetHeight}px`);
  pad();
  if ("ResizeObserver" in window) {
    resize = new ResizeObserver(pad);
    resize.observe(banner);
  }
}

if (id) {
  for (const b of document.querySelectorAll("[data-consent-open]")) {
    b.hidden = false;
    b.addEventListener("click", () => {
      showBanner();
      opener = b;
      banner?.querySelector("button")?.focus();
    });
  }
  const choice = stored();
  if (choice === true) startAnalytics();
  else {
    // Declined, expired or never asked: no analytics cookie may stay behind.
    deleteCookies();
    if (choice === null && !gpc()) showBanner();
  }
  // A choice made in another tab applies here too.
  addEventListener("storage", (e) => {
    if (e.key !== KEY) return;
    hideBanner();
    if (stored() === true) startAnalytics();
    else stopAnalytics();
  });
}
