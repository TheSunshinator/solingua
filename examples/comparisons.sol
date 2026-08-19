getMedian -> {
    parameters(
        a,
            is value, type Integer,
        b,
            is value, type Integer,
        c,
            is value, type Integer,
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
    is function, returns Integer

main -> {
    result -> getMedian(5, 3, 8)
        is value, type Integer
    printLine(result)
}
    is function, returns nothing
