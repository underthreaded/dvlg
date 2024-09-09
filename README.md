# dvlg

`dvlg` is a plaintext markup language for keeping a continuous developer log.
The syntax is intentionally simple to allow free-flow of information from your brain onto the page.
We created dvlg instead of using an existing markup format to allow us to bake in certain semantic constructs for which tooling can be built around.

## The language

The main constructs of the language include:
- Date headers, for segmenting notes by date
- TODO's
- TIL (Today I learned)
- QTS (Question to self)
- Calendar Entries, for tracking upcoming events
- General Notes

### Date Headers

```
@YYYY-MM-DD
```

### TODO's
Everyone knows what todo's are.
```
- [ ] An uncompleted TODO
- [/] An in progress TODO
- [x] A completed TODO
- [-] A dropped TODO
```

### TIL's
Short for Today I Learned. Keeping a log of these is great for tracking your progress or just interesting facts.
```
! Today I learned about dvlg
```

### QTS's
Sometimes a question occurs to you, and you don't have time to address it right now.
Note it down, come back to it later and turn it into a TIL.

This is pretty much a special case of a todo, except you don't need to add extra fluff in your description of the todo

```
? Is this language useful at all
```

The TODO version of this would be
```
- [ ] learn if this langguage is useful at all?
```

Which is, of course, too much typing.

### Calendar Entries
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

### General Notes
For when you want to make a note for a project you're working on, thinking via typing etc...

```
> This is a plain untagged note
tag> this is a tagged note
tag1/tag2> this is a multi tag note
```

### Multiline notes
When you want to take longer form notes, just keep writing on newlines.
As long as you don't use the above syntax your notes should apply to the preceding constructs (todo/til/qts/note/calendar/etc...)

```
- [ ] create video for showing off dvlg

The video should be short with no fluff.
Probably keep short and emphasise how we want to keep users typing words and focusing on the content not the process.

- [ ] Do this other thing
```

## Why
This is not a competitor to the almighty Org Mode nor a replacement for any tool in particular.
We wrote this because we weren't keeping enough notes in our day to day.
And wanted a non-obtrusive language which encoded the semantic meaning we wanted into it.
In theory allowing post-processing tooling to build on top of the language, and multiply its power.

This is a markup format specifically designed for append dominant writing.
Append dominant writing we feel is what we need to most efficiently get thoughts out of our brain and saved for future value.