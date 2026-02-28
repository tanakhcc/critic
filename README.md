# critic
This is a webapp to enable digital textual criticism.

## How to use
Build critic (or use the dockerfile) and run it agains a database.
- setup OAuth against your github organization

Ingest these basic building blocks:
- languages (via the web-UI)
- the versification scheme and base corpus (at the moment, this has to be done directly through the db)
- the models (via the web-UI)

Then, your normal project administrators can upload manuscripts, edit their metadata.
And all users can start transcribing.

## Further Notes on Setup and the basic structure
### Indexing
The uploaded images are run through kraken with the models you supply.
First, the lines are identified. After humans have an opportunity to edit these lines,
the lines are used to recognize text on them.
This yields some recognized text. We then search this recognized text in the entire base corpus.

#### Base Corpus
The base corpus currently has to be ingested directly into the DB.
TODO: add the ingestion scripts used for MAPM in this repo.

#### Versification Schemes
Since different manuscripts may have their verse breaks (if any) in different places, there may be different versification schemes.
Each such scheme consists of a list of tuples (id, name) for each verse. For the hebrew bible, this may be `(1, 'Genesis 1:1')` and so on.
While ingesting it is important that the verse-ids are generated in the correct order, and you may choose to give their representation some meaning, e.g.
`<book-nr>-<chapter-nr>-<verse-nr>` or the like. This is not required, but the ids MUST be monotonously increasing in the natural text order.
This constraint is important because it facilitates the full text search used while indexing.


## How to build

### tailwind 4.1.11
use in critic subdirectory (the main app code)
```
./tailwindcss -i style/input.css -o style/output.css --watch
```

### cargo leptos serve
Use in the main directory (the workspace dir)
```
cargo leptos serve
```
Use the `--release` toggle for release.

## Reverse Proxying critic
You need to set a relatively high `client_max_body_size` (for `nginx`).
This is currently `150m` for endpoints under `/upload`. Other paths do not need large `client_max_body_size`.


# TODO random small features
## Allow deletion of MSs, Pages, Languages, Models with a very loud warning message that that will delete human work
## Nicer optics for the admin panel
## Help overlays for
### Models
### Languages

# TODOs - Redesign for visual Transcription Editor
## base corpus search
### Write primitives
#### cleanse marked corpus
- takes streamed formatted text and turns it into
    1. The cleansed pure text (only equality alphabet, no anchors, pure basetext)
    2. A indexmap [(idx in pure text, block-idx+string-idx in formatted text)]

#### fulltext index generation
- takes a chunk (arbitrary length, e.g. an entire book) of critic-tei-xml
- breaks it into appropriate blocks
- inserts the xml form into the db
- cleanses them and inserts the clean form into the FTS index
    - contains the block ID in the document

#### search for a line in FTS
- create the query "ALLOF(FUZZY(w) for w in line)" and run it against the index

#### last and next blocks
- requires verse ordering to find the last and next block

#### Forward-completion
GIVEN:
- a match of one line to a base corpus block
    - given as
        - block id and clean-idx in that block (start)
        - block id and clean-idx in that same block (end)
- a `vec<lines>`
FIND:
- matches for the other lines (they potentially span multiple blocks)
ALGORITHM:
- if the block is not exhausted, fuzzy match word-for-word backwards (and forwards) through the lines
    - ideally, this simply matches
    - when it does not match, early return with None
    - when the block becomes exhausted before the backward(forward) search ends, load the next block with the last primitive

#### find basetext for lines on a page
- iterate over lines, FTS them
- on the first match, stop; Forward-complete the remainder of the page
    - if this fails: continue FTS on the next line
- if no line fits (or some fit FTS, but Forward-completion does not work):
    - instead just return the raw ocr text found

#### run basetext finding in the background
- endless loop; use the example from minification
- select a page that should segment or ocr
    - run it

## redo image editor
### remove layout layer
- we may want to force redo the layout analysis when the model changes
    - add a button in the editor
- for recognition, we can simply rerun the model at any time, because it is only used for block editor seeding and never overwrites human effort
### shows the image, zoomable and scrollable
- right margin shows associated objects to a baseline (recognized text, transcriptions, ...)
### shows baselines
#### these can be edited
- set type (main, marginalia)
- set start and end (pixel locations)
- when this is edited, set a flag in the DB to show that this has been modified by hand
### for each baseline, show associated information
#### type
#### automatic polygonalization
- can not be edited
- maybe do not show, since humans need not interact with it
#### automatically recognized text
#### automatically assigned base text
#### transcriptions (manual)
#### reconciliations (manual)


## Automatically (OCR) find out which part of the base text belongs to which line
If this does not work well, this entire approach will not work

## Visual Transcription Editor
### Can we add position information to TEI
Instead of div/div/div for page/col/line, we would use
- surface
- zone
- line
Alle haben `@n`, `@lang` und `@id` (sollte eh von n auf id umsteigen) und `@{coordinated}`, also können wir die Positionsdaten angeben.
Wir können `<graphic>` nutzen, um das richtige Bild zu verlinken.
Wir könnten sogar `@type` nutzen, um marginalia (Masorah, ...) speziell zu markieren (type="marginalia", normale Blöcke sind type="column")

# TODOs - next
## correctly do grapheme clustering - lacuna size calculation does not work for hebrew
## Add Button for anchor
## Reuse SVGs instead of pushing them one-by-one
## set rtl based on language
## website redesign einarbeiten
## Should render XMl nicely for the XML editor start value
## Allow changing MS language in admin interface
## Overview of required transcriptions
- should contain a direct link to the image

## Admin page for adding manuscripts
### page edit
    - this can probably wait a bit
- change name
- upload new image
- link to fullsize image
- show preview image

# TODOs - Editor
## automatically set lr/rl based on language

## Output styles
### Render to some html that is easily digestible
### allow copying the produced xml
    - both in a new component, that has three tabs - the editor is one of them

## Merge Blocks
- up and down
- das ist nicht immer erlaubt - ggf ist es besser, das einfach nicht zu erlauben und automatisch zu machen??

# TODOs - General
## setup docker for code deployment later

## create admin pages for
### editing versification schemes

## Create user-facing pages for
### transcribing
- plus a large editor
- publish transcription
### reconciliation
- this will require the reconciliation editor
- this in turn will require the reconciliation differ
### overview of required transcriptions
- search bar, links to transcribe/source
### overview of required reconciliations
- search bar, links to reconcile/source
### landing page

# TODOs - actual website
High level landing page for tanakhcc.org

# TODOs - diff
We will need a good multi-diff algorithm
take the idea from multidiff:
- use wu-diff-by-hash on two inputs
- create a mapping "content"-"present-in-inputs" like in multidiff

This multi-diff will be used in collation, but also during reconciliation

# TODOs - auto-indexing
We need a way to call into kraken to use their already trained models
- crate `pyo3` can do this

# TODOs - versification
## Layer 1 - Human-Readable to Order
convert a String (1Kg 2:14) into the verse number in the scheme
- this takes as config a list of books, with list of chapters, with nr of verses
- maybe we can auto-derive this???
    - it will be difficult to write out these lists beforehand
    - humans only operate before layer1 (with hr-values)
    - so if new verses are found later, they can simply be added at the next layer
    - this does mean that human-readable <-> order is not trivial
    - instead, it assumes BHS as a start, then maps individual inputs (e.g. PS 150:1) to individual new verse ids (i.e. a verse id that is larger then the largest verse ID in BHS)
## Layer 2 - Order to universal verse id
A bimap between the schemes order and BHSs order
- `bimap` crate exists for this
- take the identity by default
- wir speichern dann das in der DB (lassen es den user angeben):
    - "The content that BHS calls a-b is in this MS RIGHT AFTER what this MS calls verse c, which contains the same content as the verse also called c in BHS"

# TODOs - import
## WLC data
- just parse, transform to our XML schema, dump as a single file

# TODOs - Branches
## master
- the default branch that end users see and pull the data from
- used for releases
    - releases are tagged commits
- high branch protection, forced CI, ...

## `rec/<source>/<user>`
- active reconciliations that `<user>` wants to be final for `<source>`
- highly incentivise users to not open new reconciliations when ones already exist for a source
- merged into `master` when accepted

# TODOs - Workflow
- The user creates a transcription (using the editor)
    - one version (current) for each source and user is kept
- The user can "publish" a transcription
    - this adds an older version for that user and source
    - all older versions are visible to everyone
- Another or the same user can then create Reconciliations
    - Select a source (a single page/image)
    - all newest transcriptions for that source will be displayed
    - create one reconciled version (we need a new editor for this)
        - always pick the majority view for blocks by default
        - allow picking blocks from different versions (copy them over from one displayer into the final editor)
        - allow a full editor for the final version
    - create a new branch on top of master ("rec/source/user"), setting the transcript file for that source
    - create a MR from this branch onto some working branch ("critic/accepted"), with a message containing:
        - (ignore blocks that are equal)
        - how often was majority decision taken
        - how often was minority decision taken
        - how often was a completely different decision taken
- Releases then take multiple branches (e.g. for one manuscript, or for one logical part of text)
    - all these branches are merged into one big feature branch
    - that branch is rebased onto master

## Q&A
Using normal github issues

## Release Versions
Using normal github releases

## Should we host a matrix server as well for better chats?
Would be nice, but require a central LDAP for auth management

## Manuscripts, Pages
- Manuscripts = a group of folios with meta-information (representable in TEI format)
- Pages = File on the OS level (a single xml file)

# Source of truth
Source of truth is mixed.
DB is used authoritatively for:
- session stores (local sqlite)
- current state of published files
github is used authoritatively for everything else:
- actual transcription data (after reconciliation)
- (source metadata - this is part of the XML files for transcriptions)

## auto-rebuild
- DB is rebuilt every now and then (daily??) from github
    - we check consistency of github by building into a dev-db
    - if that works, we build into the actual db

## consistency check
- check that all xml files are parsable in our subscheme
- check that all metadata for a source is consistent

