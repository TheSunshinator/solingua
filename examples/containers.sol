Email -> {
    parameters(
        identifier,
            is value, type String, instance scope,
        domain,
            is value, type String, instance scope,
    )

    value -> "\(identifier)@\(domain)"
        is value, type String, instance scope,
}
    is container, project scope,

main -> { parameters()
    email -> Email("john.doe", "lingua.sol")
        is value, type Email, local scope,

    printLine(email.value)
}
    is function, returns nothing, project scope,
