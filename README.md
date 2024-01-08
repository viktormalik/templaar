# Templaar

Templaar is a simple tool for Linux for creating text files from templates.

Usage is based on two commands:
- `templaar new` - creates a new template file
- `templaar take` - finds a template file and creates a new file from it

Both commands open the created file in the default system editor (taken from
`$EDITOR` env var).

Templates are stored as hidden files named `.<TEMPL>.aar`. When searching for
templates, Templaar starts from the current directory and recursively proceeds
to its parent directories, until a template is found.

It is also possible to create a global template in `~/.config/templaar/`. This
is done using the `--global` option of the `new` command and global templates
are used when no template is found in the current or parent directories.

Full synopsis of commands:

```
Usage: templaar new [NAME]

Arguments:
  [NAME]  Name of the template

Options:
  -g, --global  Make the template global
  -h, --help    Print help
```
```
Usage: templaar take [OPTIONS] [NAME]

Arguments:
  [NAME]  Name of the created file

Options:
  -t, --template <TEMPLATE>  Use specific template
  -h, --help                 Print help
```
