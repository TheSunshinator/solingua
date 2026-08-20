getMedian -> {
    parameters(
        a
            is value, type Integer, local scope,
        b
            is value, type Integer, local scope,
        c
            is value, type Integer, local scope,
    )
    return if {
        a < b -> if {
            b < c -> b
            a <= c -> c
            else -> a
        }
        else -> if {
            b > c -> b
            a >= c -> c
            else -> a
        }
    }
}
    is function, returns Integer, project scope,

main -> {
    parameters()
    result -> getMedian(5, 3, 8)
        is value, type Integer, local scope,
    printLine(result)
}
    is function, returns nothing, project scope,
