max -> {
    parameters(
        a
            is value, type Integer, local scope,
        b
            is value, type Integer, local scope,
    )
    return if {
        a > b -> a
        else -> b
    }
}
    is function, returns Integer, local scope,

main -> {
    result -> max(5, 3)
        is value, type Integer, local scope,
    printLine(result)
}
    is function, returns nothing, local scope,
