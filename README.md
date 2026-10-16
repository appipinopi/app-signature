# App Signature

appipinopiのアプリに共通で入れる、開発者の署名と公式リンクです。

**Made by appipinopi ↗** をクリックすると開発者のGitHubへ、**公式サイト ↗** をクリックするとアプリのサイトへ移動します。専用のプロフィールページは不要です。

**[私が作ったもの ↗](https://portal.88soundboard.workers.dev/)** から、制作したアプリのポータルを開けます。この共通リンクは `creator.json` の `worksUrl` で管理し、各アプリにも表示します。

## 構成

- `creator.json`: 全アプリで使う開発者名とGitHub URL。
- `web/app-signature.js` / `.css`: フレームワークに依存しない表示部品。Webサイト、Electron、WebViewで使用できます。
- Rust crate `app-signature`: 設定の読み込み、リンクIDの解決、OSの既定ブラウザーで開く処理。Windows / macOS / Linuxに対応します。
- `examples/tickreel.json`: アプリごとに差し替える設定例。

表示のためのサーバー接続・解析・端末ID・自動更新はありません。部品と設定はアプリへ同梱します。外部ページを開くのはユーザーがリンクを押した時だけです。

## Rustアプリへ組み込む

`Cargo.toml` に追加します。リリースではタグに対応するコミットを `rev` で固定してください。

```toml
[dependencies]
app-signature = { git = "https://github.com/appipinopi/app-signature", tag = "v0.1.0" }
```

アプリ内に次のJSONを保存します。開発者情報は共通の `creator.json` が初期値になります。

```json
{"appName":"My App","links":[{"id":"website","label":"公式サイト","url":"https://example.com/"}]}
```

```rust,ignore
let signature = app_signature::Signature::from_json(include_str!("../assets/app-signature.json"))?;
signature.open("website")?;
signature.open("creator")?;
```

WebViewでは `WEB_SCRIPT` と `WEB_STYLE` をローカルのJS/CSSとして配信できます。WebViewからは `website` / `creator` のような登録済みIDだけを受け取り、Rust側で `open` してください。WebView自体の外部ナビゲーションは許可する必要がありません。

## Web / Electron / WebViewへ組み込む

GitHubのソースから `web` の2ファイルをアプリへ同梱するか、npmのGit依存として追加します。npmレジストリへの公開は不要です。

```json
{"dependencies":{"@appipinopi/app-signature":"github:appipinopi/app-signature#v0.1.0"}}
```

```html
<link rel="stylesheet" href="./app-signature.css">
<div id="creator-signature"></div>
<script type="module">
  import { mountSignature } from './app-signature.js';
  mountSignature(document.getElementById('creator-signature'), {
    signature: {
      appName: 'My App',
      creator: { name: 'appipinopi', url: 'https://github.com/appipinopi' },
      links: [{ id: 'website', label: '公式サイト', url: 'https://example.com/' }]
    }
  });
</script>
```

通常のWebではリンクを新しいタブで開きます。Electron/WebViewでは `onOpenLink: id => bridge.openPublicLink(id)` を渡して外部ブラウザーへ接続します。開けない場合はエラーを表示し、再試行できます。`destroy()` で表示を取り外せます。

独自の外観では `--signature-color`、`--signature-border`、`--signature-error` を上書きできます。ネイティブUIでは同じ `Signature` の情報を使って自由に表示できます。

## 更新

開発者名・GitHubは共通の `creator.json`、各アプリのサイトは各アプリ内のJSONで変更します。共通部品の新版をタグ付けし、各アプリで依存バージョンを上げて配布します。表示を遠隔操作する仕組みはありません。

## 検証

```sh
cargo test --locked
npm test
```

MIT License。署名はアプリ画面の開発者表示です。Windowsのコード署名や証明書による作者認証とは別のものです。
