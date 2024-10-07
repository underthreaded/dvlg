# dvlg

![](dvlg.png)

`dvlg` is a plaintext markup language for keeping a continuous log of work, dvlg is short for devlog and is pronounced "devlog".
Devlog being short for development log, not developer log, as it is not a language only for developers.
`dvlg` as a language is not optimised for reading by people other than the owner of the log, as it doesn't encourage spending time tidying the formatting.
The syntax is intentionally simple to allow free-flow of information from your brain onto the page.
While it does support a subset of markdown to be used within it's notes, we do not generally encourage it's use - dvlg is for getting ideas down and exploring them, as a step before preparing them to be shared.

We created dvlg instead of using an existing markup format to allow us to bake in certain semantic constructs for which tooling can be built around.

**Note `dvlg` is in alpha, the format is still subject to change while we collect feedback.**

## The language

The main constructs of the language include:
- Date headers, for segmenting notes by date
- General Notes
- TODO's
- Ideas
- TIL (Today I learned)
- QTS (Question to self)
- Calendar Entries, for tracking upcoming events

### Date Headers
A system for keeping track of when notes were made, we don't go finer grained than on a days basis.
```
@YYYY-MM-DD
```

The space under a date tag is automatically inferred to be a general untagged note (which you'll read more about below.)

### General Notes
`dvlg` is made for making different kinds of notes.
The simplest type of note you can take is the general note.

```
/ This is a plain untagged note
/tag/ this is a tagged note
/tag/subtag/ tags can have a path to allow you to categorise notes in a hierarchy
```

#### Multiline notes
All notes are not limited to one line.
For any note you take, just keep writing on newlines.
As long as you don't use the above syntax your notes should apply to the preceding notes (of any note type).


```
/ video for showing off dvlg

The video should be short with no fluff.
Probably keep short and emphasise how we want to keep users typing words and focusing on the content not the process or the formatting.

/ Do this other thing

- some
- markdown
- list
- because you can
```

When you do this you can think of the text inline with the note marker (for general notes the `/`) as a title, and the rest of the notes the body of the note.

### TODO's
Everyone knows what todo's are.
```
- [ ] An uncompleted TODO (if left blank priority 5, with an allowed range of 0-9, 0 being most important)
- [2] An uncompleted TODO with priority 2
- [/] An in progress TODO
- [x] A completed TODO
- [-] A dropped TODO
- [1m] A remind me later TODO for 1 month available suffixes: d, w, m, q, y. For when you want to drop something but be reminded about it after a given amount of time.
```

### TIL's
Short for Today I Learned. Keeping a log of these is great for tracking your progress or just interesting facts.
```
! Today I learned about dvlg
```

### QTS's
Sometimes a question occurs to you, and you don't have time to address it right now.
Note it down, come back to it later and turn it into a TIL.

This is pretty much a special case of a todo, except you don't need to add extra fluff in your description of the todo, and tracking the knowledge you know you don't know can sometimes come in handy!

```
? Is this language useful at all
```

The TODO version of this would be
```
- [ ] learn if this langguage is useful at all?
```

Which is, of course, too much typing.

Once you have answered or otherwise resolved this question, you can answer it by adding an answer note below, like so

```
? Is this language useful at all
?! Takes a bit of getting used to, but I like it!
```

### Calendar Entries
Calendar notes are for noting down important future dates.
```
[YYYY-MM-DD] Ship big feature
```
or with a time
```
[YYYY-MM-DD HH:MM] Check on intern
```
or with a time and duration
```
[YYYY-MM-DD HH:MM-HH:MM] Big important meeting to avoid
```


## Why
This is not a competitor to the almighty Org Mode nor a replacement for any tool in particular.
We wrote this because we weren't keeping enough notes in our day to day.
And wanted a non-obtrusive language which encoded the semantic meaning we wanted into it.
In theory allowing post-processing tooling to build on top of the language, and multiply its power.

This is a markup format specifically designed for append dominant writing.
Append dominant writing we feel is what we need to most efficiently get thoughts out of our brain and saved for future value.


## The Tooling

Currently, we have a very simple cli tool which allows simply filtering the different types of notes in your document.

This makes it easy to check your todo's for example:

```
dvlg your.dlvg todo
```

### Building the tooling
You just need `rustc`, this is stdlib only.

```
rustc dvlg.rs
```