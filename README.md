# gdb-devtools

## LSP client setup

#### vim 

Setup vimrc to identify files with `gdb` extension as filetype gdb.

```
autocmd BufNewFile,BufReadPost *.gdb set filetype=gdb
```

Then configure your LSP client plugin of choice to use `gdbls`.

```
# coc-settings.json
{
	"languageserver": {
		"gdb": {
			"command": "~/workspace/gdb-devtools/target/debug/gdbls",
			"filetypes": ["gdb"]
		}
	}
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
