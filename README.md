# YTM

YouTube Memories Viewer. Simple application for browsing your YouTube history data

![demo.gif](/assets/demo.gif)

## Installation

You need rust toolchain to build it yourself

```shell
cargo install --git https://github.com/MarrieMitsu/ytm
```

## Usage

First you must export your YouTube data in JSON format from [How to download your Google data](https://support.google.com/accounts/answer/3024190?hl=en). There's also example history [data](./data) which you can use for quick demo

Then simply run

```shell
ytm --file watch-history.json
```

It will run a local web server
