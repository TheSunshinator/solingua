main -> {
    parameters()
    a -> 10
        is value, type Integer, local scope,
    b -> 3
        is value, type Integer, local scope,

    printLine(a + b)
    printLine(a - b)
    printLine(a * b)
    printLine(a / b)
    printLine(2 + 3 * 4)
    printLine(a * 2 + b)
}
    is function, returns nothing, project scope,
