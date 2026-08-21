main -> {
    parameters()
    greeting -> "Hello world!"
        is value, type String, local scope,
    printLine(greeting)

    foo -> ""
        is variable, type String, local scope,

    foo ~~> "bar"
    printLine(foo)
}
    is function, returns nothing, project scope,
