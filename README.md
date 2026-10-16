# App Signature

appipinopiのアプリに共通で入れる、開発者の署名と公式リンクです。

**Made by appipinopi ↗** をクリックすると開発者のGitHubへ、**公式サイト ↗** をクリックするとアプリのサイトへ移動します。専用のプロフィールページは不要です。

**[私が作ったもの ↗](https://portal.88soundboard.workers.dev/)** から、制作したアプリのポータルを開けます。この共通リンクは `creator.json` の `worksUrl` で管理し、各アプリにも表示します。

## 構成

- `creator.json`: 全アプリで使う開発者名とGitHub URL。
- `web/app-signature.js` / `.css`: フレームワークに依存しない表示部品。Webサイト、Electron、WebViewで使用できます。
- Rust crate `app-signature`: 設定の読み込み、リンクIDの解決、OSの既定ブラウザーで開く処理。Windows / macOS / Linuxに対応します。
- `examples/tickreel.json`: アプリごとに差し替える設定例。

リンク設定だけをGitHubから更新できます。解析・端末IDの送信は行いません。部品はアプリに同梱し、実行ファイルやスクリプトは更新しません。外部ページを開くのはユーザーがリンクを押した時だけです。

## 「私が作ったもの」をアプリ更新なしで変更する

[creator.jsonを編集](https://github.com/appipinopi/app-signature/edit/main/creator.json)し、`worksUrl` を変更してmainへコミットします。対応するアプリは起動時と1時間ごとに取得します。全OS・全配布版で共通です。オフラインや不正な設定の場合は前回の有効な設定、初回は同梱の設定を使います。

Rustでは `LiveSignature::new(signature, cache_path)` を保持し、`snapshot()` で現在の設定を読みます。終了時に破棄すると取得スレッドも終了します。Webでは `refreshCreator(signature)` を呼び、戻り値を `mountSignature` に渡します。通常のWebで定期更新する場合は1時間ごとに再実行してください。

## Rustアプリへ組み込む

`Cargo.toml` に追加します。リリースではタグに対応するコミットを `rev` で固定してください。

```toml
[dependencies]
app-signature = { git = "https://github.com/appipinopi/app-signature", tag = "v0.2.0" }
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
{"dependencies":{"@appipinopi/app-signature":"github:appipinopi/app-signature#v0.2.0"}}
```

```html
<link rel="stylesheet" href="./app-signature.css">
<div id="creator-signature"></div>
<script type="module">
  import { mountSignature } from './app-signature.js';
  mountSignature(document.getElementById('creator-signature'), {
    signature: {
      appName: 'My App',
      creator: { name: 'appipinopi', url: 'https://github.com/appipinopi', worksUrl: 'https://portal.88soundboard.workers.dev/' },
      links: [{ id: 'website', label: '公式サイト', url: 'https://example.com/' }]
    }
  });
</script>
```

通常のWebではリンクを新しいタブで開きます。Electron/WebViewでは `onOpenLink: id => bridge.openPublicLink(id)` を渡して外部ブラウザーへ接続します。開けない場合はエラーを表示し、再試行できます。`destroy()` で表示を取り外せます。

独自の外観では `--signature-color`、`--signature-border`、`--signature-error` を上書きできます。ネイティブUIでは同じ `Signature` の情報を使って自由に表示できます。

## 更新

開発者名・GitHub・作品ポータルは共通の `creator.json`、各アプリの公式サイトは各アプリ内のJSONで変更します。LiveSignature対応アプリでは、開発者のリンク変更だけならアプリの再配布は不要です。共通部品そのものの変更は各アプリの依存バージョンを上げて配布します。

## 検証

```sh
cargo test --locked
npm test
```

MIT License。署名はアプリ画面の開発者表示です。Windowsのコード署名や証明書による作者認証とは別のものです。
