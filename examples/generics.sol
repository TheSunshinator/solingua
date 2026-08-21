Some -> {
    parameters(
        value
            is value, type T, instance scope,
    )
}
    is blueprint, full implementation, generics T

main -> {
    foo -> Some("Foo's value")
        is value, type Some of String, local scope,

    printLine(foo.value)
}
    is function, returns nothing, project scope,
