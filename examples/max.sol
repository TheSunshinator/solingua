max -> {
    parameters(
        a,
            is value, type Integer,
        b,
            is value, type Integer,
    )
    return if {
        a > b -> a
        else -> b
    }
}
    is function, returns Integer

main -> {
    result -> max(5, 3)
        is value, type Integer
    printLine(result)
}
    is function, returns nothing
