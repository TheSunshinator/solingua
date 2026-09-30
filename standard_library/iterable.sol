let singleton IterationEnd {
    #return() #visibility(private) #scope(project)
}
※※
let type Iterable {
    #return() #generic⟨T⟩ #extensible(open) #visibility(public) #scope(project)
    #implementation(none)

    let 𝑓 next {
        #returns(T) #generic⟨⟩ #extensible(open) #visibility(public) #scope(instance)
        #implementation(none)
    }
}
※※