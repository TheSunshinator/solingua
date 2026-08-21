main -> {
    counter -> 0
        is variable, type Integer, local scope,

    while counter < 10 {
        printLine(counter)
        counter ~~> counter + 1
    }
}
    is function, returns nothing, project scope,
