# Product website

Plain static HTML/CSS/JS. No bundler, external runtime dependencies, analytics, or remote fonts.

```sh
python3 -m http.server 4173 --directory website
bun test website/tests
```

Open http://localhost:4173. The timer is an explicitly labeled demo, with no persistence or notifications. It uses elapsed monotonic time rather than counting interval callbacks. Screenshot controls progressively enhance a real, static screenshot. All install links work without JavaScript.

`.github/workflows/pages.yml` validates and deploys on changes to `website/` on `master`, or manual dispatch. Only HTML, CSS, JS and assets are uploaded: tests, documentation and design records are not published. Pages uses the GitHub Actions source. Public URL: https://aungmyokyaw.github.io/rusty-pomodoro/.

Screenshots originate in `assets/screenshots/`; optimized WebP copies ship here. Fraunces and DM Sans are self-hosted under the SIL Open Font License; license files are included alongside their fonts.
