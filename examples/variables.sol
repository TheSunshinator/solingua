main -> {
    parameters()
    greeting -> "Hello world!"
        is value, type String, local scope,
    printLine(greeting)
}
    is function, returns nothing, project scope,
