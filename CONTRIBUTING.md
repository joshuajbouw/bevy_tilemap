# Thank You!
Thank you so much for your interest into contributing to the Bevy Tilemap project! I can't do this 
alone and better practices will always come to help excel this further. I'll keep the guidelines as 
brief as I can as they are meant to really just streamline the Pull Request process.

## Table of Contents
1. [About the Project](#about-the-project)
1. [Process](#process)
    1. [Questions](#questions?)
    1. [How to Request New Features](#how-to-request-new-features)
    1. [How to Report a Bug](#how-to-report-a-bug)
    1. [Submitting a Pull Request](#submitting-a-pull-request)
1. [Contributing Code](#contributing-code)
    1. [How to Setup the Development Environment](#how-to-setup-the-development-environment)
    1. [Development Process](#development-process)
    1. [Style Guidelines](#style-guidelines)

## About the Project
Bevy Tilemap was made to fill an obvious need in the early Bevy community for a way to render tiled
sprites. In the beginning, and at the time of this writing, this wasn't possible until now. While 
it is still early and there is a lot planned, the project is trying to solidify most of the API as
soon as possible.

## Process
### Questions?
If you have any questions do direct them to Joshua J. Bouw in the Bevy project room on Discord or
contact him directly @joshuajbouw.

### How to Request New Features
Please fill out and [submit a feature request](https://github.com/joshuajbouw/bevy_tilemap/issues/new?assignees=&labels=enhancement&template=feature_request.md&title=)
and fill out in detail exactly what you would like to have added. However, PLEASE use the search 
function first to see if the requested feature already exists. The information that we have, 
especially if you link to other projects that do have that feature, the better we can work to
implement it.

### How to Report a Bug
If you find a bug please [submit a bug report](https://github.com/joshuajbouw/bevy_tilemap/issues/new?assignees=&labels=bug&template=bug_report.md&title=).
DO fill out all the fields and DO search if your bug has already been reported or not. The more
information we have, the better. There are fields that will be there already, just fill out the 
questions.

### Submitting a Pull Request
To save time and to prevent the code quality checks from failing do run these Rust commands first:
```bash
cargo fmt +nightly --all
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features
```

The code will be reviewed if all code quality checks have been successful and it is determined that
it is in scope. Please try to not change the API, but rather add to it. If an API does need to be
changed DO [submit a feature request](https://github.com/joshuajbouw/bevy_tilemap/issues/new?assignees=&labels=enhancement&template=feature_request.md&title=) instead and explicitly state that and why. The
maintainers strive to be active and will try to respond on a daily basis in their own free time.

## Contributing Code
### How To Setup the Development Environment
There are a number of IDEs to choose from. The most popular is JetBrain's IDEA CE with the `rust` 
and `toml` plugins which can be found on [the Jetbrain's website](https://intellij-rust.github.io/).
Even better is CLion though that is paid for. CLion comes with added debugging features.

Do checkout this [post by Tremaine Eto](https://medium.com/cloud-native-the-gathering/whats-the-best-ide-for-developing-in-rust-5087d46006f5) 
which covers mostly everything you would need to know.

### Development Process
#### `master` Branch
The master branch which will solely track to latest Bevy release. The tags on the master branch will
have tags of each version that was pushed to crates.io.

#### `bevy-x.y` Branch
In-case of impending Bevy release with new features or is API breaking this will take place on a 
`bevy_x.y` branch. Once the new Bevy is released, then all those changes will be moved to master. 
There may be pre-release versions tagged on this branch for others to use. 

### Style Guidelines
All [Rust API guidelines](https://rust-lang.github.io/api-guidelines/) do apply. Do note that every
once in a bit, before a release, the [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html) 
will be done to ensure code quality fits. DO NOTE that this will NOT be entirely a factor in 
approving a pull request and will only save the maintainers time before a release. Please follow as 
a courtesy.

#### Struct Objects
* Constructors
* Getters
    * Note: TRY to avoid
    * Method with `get_<requesting>(&self, _: _)` i.e, `get_chunk(&self, index: usize)`
    * Returning with `&T` i.e, `&Chunk`
* Setters (`set_` try to avoid, must have reason)
    * Note: TRY to avoid
    * Method with `set_<requesting>(&mut self, _:_)` i.e `set_chunks(&mut self, chunks: Vec<Chunk>)`
    * Typically returns if replacing something. Will return what was replaced.
* References (`-> &str`, `-> &[u32]`, etc.)
    * Note: TRY to avoid
    * Method with `<requesting>(&self, _: _)` i.e, `tile(&self, index: usize)`
    * Returning with `&T` i.e, `&Tile`
* Mutable references
    * Note: TRY to avoid
    * Method with `<requesting>_mut(&mut self, _: _)` i.e, `tile_mut(&mut self, index: usize)`
    * Returning with `&mut T` i.e, `&mut Tile`
* Utils
    * Inner non-public methods
    * i.e updates, inner methods
* General methods
    * Generally more refined methods that process the input and stores or returns a value.
    * i.e `add_` `remove_`
* `is_` bools

#### Derives
* Do make `derives` alphabetical.

#### `Impl`s
* Do make `Impl`s alphabetical of what is being implemented with the main `Impl` 
(i.e `impl Chunk {`) last.
