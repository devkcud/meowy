The `draw_text` definition has a single argument with the type:

```meowy
<Options> : <{
    text <string>
    size <uint8>
    color <rl.Color><null>
    x <int128>
    y <int128>
}>
```

It may feel stranger and alien when compared to its C counterpart, but we assure it's the same implementation. Under the hood, we just grab from `options.*` (text, size, etc.).

Also, every `<null>` value can be safely ignored.

```meowy
rl.draw_text({
    -> text : "Congrats! You created your first window!"
    -> size : 20

    # color will use the default rl.BLACK #

    -> x : 190
    -> y : 200
})
```
