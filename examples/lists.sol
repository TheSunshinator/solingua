main -> {
    names -> List("Ben", "Peter", "May")
        is value, type List of String, local scope,

    print("Hello ")
    printLine(names.at(2))
    printLine(names.size)
}
    is function, returns nothing, project scope,
