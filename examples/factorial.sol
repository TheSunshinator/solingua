factorial -> {
    parameters(
        n
            is value, type Integer, local scope,
    )
    return if {
        n = 0 -> 1
        else -> n * factorial(n - 1)
    }
}
    is function, returns Integer, project scope,

main -> {
    parameters()
    printLine(factorial(15))
}
    is function, returns nothing, project scope,
