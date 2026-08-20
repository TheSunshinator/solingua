main -> {
    parameters()
    for i from 0 to 5 -> {
        printLine(i)
    }
        exclude last,

    for i from 1 to 5 -> printLine(i * i)
        include last,
}
    is function, returns nothing, project scope,
