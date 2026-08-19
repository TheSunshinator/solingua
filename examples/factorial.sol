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
    is function, returns Integer, local scope,

main -> {
    printLine(factorial(15))
}
    is function, returns nothing, local scope,
