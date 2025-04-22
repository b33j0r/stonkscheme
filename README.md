# `stonkscheme`

A toy scheme interpreter that aims to implement a subset of something like PowerLanguage or EasyLanguage. (I think I was mostly amused by
the name).

The goal of this was to revisit how I do Spanned<T> with nom 8. I was hoping to make an extension trait `ParserExt` that would allow me to do `(some_parser).parse_spanned(input)`.