# sv

Everything you need to build a Svelte project, powered by [`sv`](https://github.com/sveltejs/cli).

## Creating a project

If you're seeing this, you've probably already done this step. Congrats!

```sh
# create a new project
npx sv create my-app
```

To recreate this project with the same configuration:

```sh
# recreate this project
npx sv@0.15.3 create --template minimal --types ts --add prettier eslint vitest="usages:unit,component" playwright tailwindcss="plugins:typography,forms" --install npm scada-foundry-ui
```

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```sh
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```sh
npm run build
```

You can preview the production build with `npm run preview`.

> To deploy your app, you may need to install an [adapter](https://svelte.dev/docs/kit/adapters) for your target environment.


◇  What's next? ────────────────────────────────────────────╮
│                                                           │
│  📁 Project steps                                         │
│                                                           │
│    1: cd scada-foundry-ui                                 │
│    2: npm run dev -- --open                               │
│                                                           │
│  To close the dev server, hit Ctrl-C                      │
│                                                           │
│  🧩 Add-on steps                                          │
│                                                           │
│    playwright:                                            │
│      - Run npx playwright install to download browsers    │
│      - Visit /demo/playwright to see the demo page        │
│      - Run npm run test:e2e to execute the example tests  │
│                                                           │
│  Stuck? Visit us at https://svelte.dev/chat               │
│                                                           │
├───────────────────────────────────────────────────────────╯