# Templaar

Templaar is a simple tool for Linux for creating text files from templates.

Usage is based on two commands:
- `templaar new` - creates a new template
- `templaar take` - finds a template and creates new file(s) from it

Both commands open the created file(s) in the default system editor (taken from
the `$EDITOR` env var) for further editing.

There are two kinds of templates: *file* templates consisting of a single file
and *directory* templates consisting of a directory containing multiple files.

Templates are stored as hidden files/directories named `.<TEMPL>.aar`. When
searching for templates, Templaar starts from the current directory and
recursively proceeds to its parent directories, until a template is found.

It is also possible to create a global template in `~/.config/templaar/`. This
is done using the `--global` option of the `new` command and global templates
are used when no template is found in the current or parent directories.

The list of currently available templates (both local and global) can be shown
by `templaar list`.

Full synopsis of commands:

```
Usage: templaar new [NAME]

Arguments:
  [NAME]  Name of the template

Options:
  -g, --global              Make the template global
  -f, --files [<FILES>...]  Create the template from file(s).
                            In case of multiple files, the template will be a directory.
  -h, --help                Print help
```
```
Usage: templaar take [OPTIONS] [NAME]

Arguments:
  [NAME]  Name of the created file.
          Path in the case of a directory template.

Options:
  -t, --template <TEMPLATE>  Use specific template
  -h, --help                 Print help
```
```
Usage: templaar list [OPTIONS]

Options:
  -l, --local   Only list local templates
  -g, --global  Only list global templates
  -h, --help    Print help
```
