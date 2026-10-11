// LMM Genned, may or ma ynot work as intended.
/* SISR downloads helper
 * - Fetches latest GitHub release
 * - Tries to pick the best asset for the visitor (Windows/Linux + x64/arm64)
 * - Populates a big primary download button + asset list
 */

(function () {
    "use strict";

    const OWNER = "Alia5";
    const REPO = "SISR";
    const API_LATEST_STABLE = `https://api.github.com/repos/${OWNER}/${REPO}/releases/latest`;
    const API_RELEASES = `https://api.github.com/repos/${OWNER}/${REPO}/releases?per_page=20`;
    const RELEASES_URL = `https://github.com/${OWNER}/${REPO}/releases`;

    function $(id) {
        return document.getElementById(id);
    }

    function setStatus(el, msg) {
        if (!el) return;
        el.textContent = msg;
    }

    function formatPlatformArch(want) {
        const p = want.platform === "unknown" ? "your OS" : want.platform;
        const a = want.arch === "unknown" ? "" : ` ${want.arch}`;
        const s = `${p}${a}`.trim();
        if (!s) return s;
        return s.charAt(0).toUpperCase() + s.slice(1);
    }

    function bytesToMiB(n) {
        if (typeof n !== "number") return "";
        return `${(n / (1024 * 1024)).toFixed(1)} MiB`;
    }

    function normalize(s) {
        return String(s || "").toLowerCase();
    }

    async function detectPlatformArch() {
        // Best-effort: browsers vary wildly here.
        let platform = "unknown";
        let arch = "unknown";

        // Platform
        const ua = normalize(navigator.userAgent);
        const navPlatform = normalize(navigator.platform);
        const uaPlatform = normalize(navigator.userAgentData && navigator.userAgentData.platform);

        const p = uaPlatform || navPlatform || ua;
        if (p.includes("win")) platform = "windows";
        else if (p.includes("linux")) platform = "linux";
        else if (p.includes("mac") || p.includes("darwin")) platform = "darwin";

        // Arch
        // Try User-Agent Client Hints first
        try {
            if (navigator.userAgentData && navigator.userAgentData.getHighEntropyValues) {
                const v = await navigator.userAgentData.getHighEntropyValues(["architecture", "bitness"]);
                const a = normalize(v.architecture);
                const b = normalize(v.bitness);
                if (a.includes("arm")) arch = "arm64";
                else if (a.includes("x86") || a.includes("x64") || b === "64") arch = "x64";
            }
        } catch {
            // ignore
        }

        // Fallback heuristics
        if (arch === "unknown") {
            if (ua.includes("arm64") || ua.includes("aarch64")) arch = "arm64";
            else if (ua.includes("x86_64") || ua.includes("win64") || ua.includes("x64") || ua.includes("amd64")) arch = "x64";
        }

        return { platform, arch };
    }

    function scoreAsset(assetName, wantPlatform, wantArch) {
        const n = normalize(assetName);
        let score = 0;

        // Filter-ish: de-prioritize checksum/signature files
        if (n.endsWith(".sha256") || n.endsWith(".sha256sum") || n.endsWith(".sig") || n.endsWith(".asc") || n.endsWith(".txt")) {
            score -= 50;
        }

        // Platform
        if (wantPlatform === "windows") {
            if (n.includes("windows") || n.includes("win")) score += 40;
            if (n.endsWith(".msi")) score += 15;
            if (n.endsWith(".exe")) score += 10;
            if (n.endsWith(".zip")) score += 5;
        }
        if (wantPlatform === "linux") {
            if (n.includes("linux")) score += 40;
            if (n.endsWith(".appimage")) score += 20;
            if (n.endsWith(".tar.gz") || n.endsWith(".tgz")) score += 8;
            if (n.endsWith(".deb") || n.endsWith(".rpm")) score += 5;
        }
        if (wantPlatform === "darwin") {
            if (n.includes("mac") || n.includes("darwin") || n.includes("osx")) score += 40;
            if (n.endsWith(".dmg") || n.endsWith(".pkg") || n.endsWith(".zip")) score += 10;
        }

        // Arch
        if (wantArch === "arm64") {
            if (n.includes("arm64") || n.includes("aarch64")) score += 30;
            if (n.includes("arm")) score += 5;
        }
        if (wantArch === "x64") {
            if (n.includes("x64") || n.includes("x86_64") || n.includes("amd64")) score += 30;
            if (n.includes("x86")) score += 5;
        }

        // Generic preference: non-source
        if (n.includes("source")) score -= 10;
        if (n.includes("src")) score -= 5;

        return score;
    }

    function pickBestAsset(assets, wantPlatform, wantArch) {
        if (!Array.isArray(assets) || assets.length === 0) return null;

        let best = null;
        let bestScore = -Infinity;

        for (const a of assets) {
            const s = scoreAsset(a.name, wantPlatform, wantArch);
            if (s > bestScore) {
                best = a;
                bestScore = s;
            }
        }

        // If we couldn't find anything that looks remotely right, bail.
        if (bestScore < 5) return null;
        return best;
    }

    function renderAssetsList(assets, listEl) {
        if (!listEl) return;
        listEl.innerHTML = "";

        for (const a of assets) {
            const li = document.createElement("li");
            const link = document.createElement("a");
            link.href = a.browser_download_url;
            link.textContent = a.name;

            const meta = document.createElement("span");
            meta.style.opacity = "0.85";
            meta.style.marginLeft = "0.5rem";
            meta.textContent = a.size ? `(${bytesToMiB(a.size)})` : "";

            li.appendChild(link);
            li.appendChild(meta);
            listEl.appendChild(li);
        }
    }

    async function fetchJson(url) {
        const res = await fetch(url, {
            headers: {
                "Accept": "application/vnd.github+json"
            }
        });
        if (!res.ok) throw new Error(`GitHub API returned ${res.status}`);
        return res.json();
    }

    function pickLatestPrerelease(releases) {
        if (!Array.isArray(releases)) return null;
        for (const r of releases) {
            if (r && r.draft) continue;
            if (r && r.prerelease) return r;
        }
        return null;
    }

    async function initDownloadsPage() {
        const stableBtn = $("sisr-download-stable");
        const prereleaseBtn = $("sisr-download-prerelease");
        const status = $("sisr-download-status");
        const latestEl = $("sisr-download-latest");
        const assetsEl = $("sisr-download-assets");

        // Not on the downloads page.
        if (!status || !assetsEl || (!stableBtn && !prereleaseBtn)) return;

        if (stableBtn) {
            stableBtn.setAttribute("href", RELEASES_URL);
            stableBtn.textContent = "Download latest stable (auto)";
        }
        if (prereleaseBtn) {
            prereleaseBtn.setAttribute("href", RELEASES_URL);
            prereleaseBtn.textContent = "Latest pre-release (auto)";
        }

        setStatus(status, "Loading release info…");

        let want;
        try {
            want = await detectPlatformArch();
        } catch {
            want = { platform: "unknown", arch: "unknown" };
        }

        let stableLine = "Stable: (not loaded)";
        let prereleaseLine = "Pre-release: (not loaded)";

        try {
            // Fetch stable + prerelease (in parallel)
            const [stableRelease, releases] = await Promise.all([
                fetchJson(API_LATEST_STABLE),
                fetchJson(API_RELEASES)
            ]);

            // ---- Stable
            const stableTag = stableRelease.tag_name || stableRelease.name || "latest";
            const stableHtmlUrl = stableRelease.html_url || RELEASES_URL;
            const stableAssets = Array.isArray(stableRelease.assets) ? stableRelease.assets : [];

            if (latestEl) {
                // Keep it compact, but show both.
                const pre = pickLatestPrerelease(releases);
                const preTag = pre ? (pre.tag_name || pre.name || "pre-release") : "(none)";
                const preUrl = pre ? (pre.html_url || RELEASES_URL) : RELEASES_URL;
                latestEl.innerHTML = `Latest stable: <a href="${stableHtmlUrl}">${stableTag}</a> &nbsp;|&nbsp; Latest pre-release: <a href="${preUrl}">${preTag}</a>`;
            }

            renderAssetsList(stableAssets, assetsEl);

            if (stableBtn) {
                const bestStable = pickBestAsset(stableAssets, want.platform, want.arch);
                if (!bestStable) {
                    stableBtn.setAttribute("href", stableHtmlUrl);
                    stableBtn.textContent = "Download latest stable (open release)";
                    stableLine = `Stable: couldn’t auto-pick for ${formatPlatformArch(want)} (showing assets below)`;
                } else {
                    stableBtn.setAttribute("href", bestStable.browser_download_url);
                    stableBtn.textContent = `Download latest stable for ${formatPlatformArch(want)}`;
                    stableLine = `Stable: ${bestStable.name}`;
                }
            } else {
                stableLine = `Stable: ${stableTag}`;
            }

            // ---- Pre-release
            const pre = pickLatestPrerelease(releases);
            if (!prereleaseBtn) {
                prereleaseLine = pre ? `Pre-release: ${pre.tag_name || pre.name || "pre-release"}` : "Pre-release: (none)";
            } else if (!pre) {
                prereleaseBtn.setAttribute("href", RELEASES_URL);
                prereleaseBtn.textContent = "Latest pre-release (none)";
                prereleaseLine = "Pre-release: none found";
            } else {
                const preHtmlUrl = pre.html_url || RELEASES_URL;
                const preAssets = Array.isArray(pre.assets) ? pre.assets : [];
                const bestPre = pickBestAsset(preAssets, want.platform, want.arch);
                if (!bestPre) {
                    prereleaseBtn.setAttribute("href", preHtmlUrl);
                    prereleaseBtn.textContent = "Latest pre-release (open release)";
                    prereleaseLine = `Pre-release: couldn’t auto-pick for ${formatPlatformArch(want)}`;
                } else {
                    prereleaseBtn.setAttribute("href", bestPre.browser_download_url);
                    prereleaseBtn.textContent = `Latest pre-release for ${formatPlatformArch(want)}`;
                    prereleaseLine = `Pre-release: ${bestPre.name}`;
                }
            }

            setStatus(status, `${stableLine}\n${prereleaseLine}`);
        } catch (err) {
            if (stableBtn) {
                stableBtn.setAttribute("href", RELEASES_URL);
                stableBtn.textContent = "Download latest stable (open releases)";
            }
            if (prereleaseBtn) {
                prereleaseBtn.setAttribute("href", RELEASES_URL);
                prereleaseBtn.textContent = "Latest pre-release (open releases)";
            }
            setStatus(
                status,
                `Failed to load release data from GitHub. Open All releases instead.\n(${String(err && err.message ? err.message : err)})`
            );
        }
    }

    function hookInit(fn) {
        // mkdocs-material instant navigation support
        if (typeof window.document$ !== "undefined" && window.document$ && typeof window.document$.subscribe === "function") {
            window.document$.subscribe(fn);
            return;
        }
        if (document.readyState === "loading") {
            document.addEventListener("DOMContentLoaded", fn);
        } else {
            fn();
        }
    }

    hookInit(initDownloadsPage);
})();
