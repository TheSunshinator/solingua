※※
Monad⟨T⟩ — A computational context that supports sequential composition.

Contract:
    unit(value)  — Wraps a value in the monad (static factory)
    bind(transform) — Applies a transformation that returns a new monad

Laws:
    1. Left identity:  unit(a) → bind(f) = f(a)
    2. Right identity: m → bind(unit) = m
    3. Associativity:  m → bind(f) → bind(g) = m → bind(x ↦ f(x) → bind(g))
※※

let type Monad {
    #return() #generic⟨T⟩ #extensible(open) #visibility(public)
    #scope(project) #implementation(none);

    instance
    let 𝑓 bind {
        #return(Self⟨T⟩) #generic⟨⟩ #extensible(open) #visibility(public)
        #scope(instance) #implementation(none);

        parameters
        let data transform {
            #return(Self⟨T⟩) #mutable(false) #scope(local) #implementation(full);
        }
    }

    project
    let 𝑓 unit {
        #return(Self⟨T⟩) #generic⟨T⟩ #visibility(public)
        #scope(project) #implementation(none);

        parameters
        let data wrapped {
            #return(T) #mutable(false) #scope(local) #implementation(full);
        }
    }
}
