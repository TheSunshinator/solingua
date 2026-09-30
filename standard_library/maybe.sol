※※
∃?⟨T⟩ — A value that may or may not exist. Implements Monad.

Variants:
    ∃(value)  — There Exists: wraps a value
    ∄         — Does Not Exist: no value

Usage:
    let data name {
        #return(∃?⟨String⟩) #mutable(false) #scope(local) #implementation(full);
        initially ∃("Krokmou")
    }

    let data empty {
        #return(∃?⟨String⟩) #mutable(false) #scope(local) #implementation(full);
        initially ∄
    }

    printLine(name → exists());          ※ true
    printLine(name → value());           ※ "Krokmou"
    printLine(empty → exists());         ※ false
    printLine(empty → valueOr("???"));   ※ "???"
※※

let type ∃? {
    #return(Monad⟨T⟩) #generic⟨T⟩ #extensible(sealed) #visibility(public)
    #scope(project) #implementation(partial);

    instance
    let 𝑓 bind {
        #return(Self⟨U⟩) #generic⟨U⟩ #extensible(open) #visibility(public)
        #scope(instance) #implementation(none) #contract(Monad);

        parameters
        let data transform {
            #return(Self⟨U⟩) #mutable(false) #scope(local) #implementation(full);
        }
    }

    project
    let 𝑓 unit {
        #return(Self⟨T⟩) #generic⟨T⟩ #visibility(public)
        #scope(project) #implementation(full) #contract(Monad);

        parameters
        let data wrapped {
            #return(T) #mutable(false) #scope(local) #implementation(full);
        }

        body
        return ∃(wrapped);
    }

    let type ∃ {
        #return(∃?⟨T⟩) #generic⟨T⟩ #extensible(closed) #visibility(public)
        #scope(project) #implementation(full);

        parameters
        let data wrapped {
            #return(T) #mutable(false) #scope(instance) #implementation(full);
        }

        instance
    }

    let type ∄ {
        #return(∃?⟨T⟩) #generic⟨T⟩ #extensible(closed) #visibility(public)
        #scope(project) #implementation(full);

        parameters
        instance
    }
}
