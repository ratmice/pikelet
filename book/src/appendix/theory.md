# Theory

A formalization of the semantics for type checking and normalizing Pikelet.

## Contents

- [Introduction](#introduction)
  - [Notation](#notation)
  - [Where is the soundness proof?](#where-is-the-soundness-proof)
- [Syntax](#syntax)
  - [Raw terms](#raw-terms)
  - [Terms](#terms)
  - [Values](#values)
  - [Contexts](#contexts)
- [Semantics](#semantics)
  - [Elaboration](#elaboration)
  - [Normalization](#normalization)
  - [Type checking](#type-checking)
  - [Type inference](#type-inference)
  - [Pattern matching](#pattern-matching)
  - [Type checking of patterns](#type-checking-of-patterns)
  - [Type inference of patterns](#type-inference-of-patterns)

## Introduction

At its core, Pikelet is a dependently typed lambda calculus with a stratified
universe hierarchy.

> **Note:**
> This document is intended for those who are interested in looking deeper into the formal foundations of Pikelet.
> You _don't_ need to understand this for general use of Pikelet, so feel free to skip this document if that is easier.
> We will however make an effort to explain some of the notation we use here, and point to resources that might help if this piques your curiosity!

### Notation

We use a combination of some [BNF][bnf]-style syntax definitions with
[natural deduction](natural-deduction) rules to define our language. This
combination of notation is sometimes referred to as _computer science
metanotation_ and is, alas, a little hard to pin down [as conventions vary][guy-steele-presentation]
between papers and authors. The general rules stay the same however, and once
you learn to read them they are much more succinct than an actual implementation
could be, and are an invaluable tool for quickly getting a high-level overview
of a programming language's semantics.

> **TODO:**
> Describe BNF, natural deduction rules, overbars, variable binding, etc.

Some handy links:

- [Crash Course on Notation in Programming Language Theory](http://siek.blogspot.com.au/2012/07/crash-course-on-notation-in-programming.html)
- [A practitioner’s guide to reading programming languages papers](https://blog.acolyer.org/2018/01/26/a-practitioners-guide-to-reading-programming-languages-papers/)
- [A path to enlightenment in Programming Language Theory](http://steshaw.org/plt/)

[bnf]: https://en.wikipedia.org/wiki/Backus%E2%80%93Naur_form
[natural-deduction]: https://en.wikipedia.org/wiki/Natural_deduction
[guy-steele-presentation]: https://www.youtube.com/watch?v=7HKbjYqqPPQ

### Where is the soundness proof?

Here we are only defining the rules of our language's type checking and
evaluation. Further work needs to be done to verify that our system actually
satisfies certain interesting [type soundness properties][type-soundness],
like progress, preservation, [strong normalization][normalization-property],
etc. If you would like to discuss this with us, please check out
[the relevant github issue][formalization-issue]!

[type-soundness]: https://en.wikipedia.org/wiki/Type_safety
[normalization-property]: https://en.wikipedia.org/wiki/Normalization_property_(abstract_rewriting)
[formalization-issue]: https://github.com/pikelet-lang/pikelet/issues/39

## Syntax

### Raw terms

\\[
% Haskell-style append https://tex.stackexchange.com/questions/4194/how-to-typeset-haskell-operator-and-friends
\\newcommand\doubleplus{+\kern-1.3ex+\kern0.8ex}
% Small caps https://github.com/mathjax/MathJax-docs/wiki/Small-caps-%5Ctextsc-in-MathJaxx
\\def\sc#1{\dosc#1\csod}
\\def\dosc#1#2\csod{{\rm #1{\small #2}}}
\\
\\newcommand{\rule}[3]{ \dfrac{ ~~#2~~ }{ ~~#3~~ } & \Tiny{\text{(#1)}} }
\\
\\DeclareMathOperator{\max}{max}
\\DeclareMathOperator{\field}{field}
\\DeclareMathOperator{\fieldty}{fieldty}
\\DeclareMathOperator{\fieldsubst}{fieldsubst}
\\DeclareMathOperator{\Match}{\sc{MATCH}}
\\
% Judgments
\\newcommand{\eval}[3]{ #1 \vdash #2 \hookrightarrow #3 }
\\newcommand{\check}[4]{ #1 \vdash #2 \uparrow #3 \leadsto #4 }
\\newcommand{\infer}[4]{ #1 \vdash #2 \downarrow #3 \leadsto #4 }
\\newcommand{\match}[3]{ \Match(#1,#2) \Longrightarrow #3 }
\\newcommand{\checkpat}[5]{ #1 \vdash #2 \uparrow #3 \leadsto #4 \Longrightarrow #5 }
\\newcommand{\inferpat}[5]{ #1 \vdash #2 \downarrow #3 \leadsto #4 \Longrightarrow #5 }
\\
% Metavariables
\\newcommand{\rexpr}{r} % raw expressions
\\newcommand{\rtype}{R} % raw types
\\newcommand{\rpat}{s}  % raw patterns
\\
\\newcommand{\texpr}{t} % expressions
\\newcommand{\ttype}{T} % types
\\newcommand{\tpat}{p}  % patterns
\\
\\newcommand{\vexpr}{v} % value expressions
\\newcommand{\vtype}{V} % value types
\\newcommand{\wexpr}{w} % whnf expressions
\\newcommand{\wtype}{W} % whnf types
\\newcommand{\nexpr}{n} % neutral expressions
\\newcommand{\ntype}{N} % neutral types
\\
\\newcommand{\ctx}{\Gamma} % contexts
\\
% Keywords
\\newcommand{\kw}[1]{ \mathsf{#1} }
\\
% Term and Type constructors
\\newcommand{\var}{x}
\\newcommand{\label}{l}
\\newcommand{\Type}{\kw{Type}}
\\newcommand{\Bool}{\kw{Bool}}
\\newcommand{\true}{\kw{true}}
\\newcommand{\false}{\kw{false}}
\\newcommand{\Arrow}[2]{ #1 \rightarrow #2 }
\\newcommand{\Pi}[2]{ \Arrow{(#1)}{#2} }
\\newcommand{\lam}[2]{ \kw{\lambda} #1 . #2 }
\\newcommand{\app}[2]{ #1 ~ #2 }
\\newcommand{\ifte}[3]{ \kw{if} ~ #1 ~ \kw{then} ~ #2 ~ \kw{else} ~ #3 }
\\newcommand{\case}[2]{ \kw{case} ~ #1 \left\\{ #2 \right\\} }
\\newcommand{\Record}[1]{ \kw{Record} \left\\{ #1 \right\\} }
\\newcommand{\record}[1]{ \kw{record} \left\\{ #1 \right\\} }
\\newcommand{\subst}[3]{ #1 ~ [#2 \rightarrow #3] }
\\
% Items
\\newcommand{\declItem}[2]{ #1 : #2 }
\\newcommand{\defnItem}[2]{ #1 = #2 }
\\
% Contexts
\\newcommand{\emptyCtx}{ \varnothing }
\\newcommand{\composeCtx}[2]{ #1 \sim #2 }
\\newcommand{\extendCtx}[2]{ #1, #2 }
\\
\begin{array}{rrll}
    \rexpr,\rtype   & ::= & \var                                & \text{variables} \\\\
                    &   | & \Type_i                             & \text{universe of types ($i \in \mathbb{N}$)} \\\\
                    &   | & ?                                   & \text{holes} \\\\
                    &   | & \Bool                               & \text{type of booleans} \\\\
                    &   | & \true ~|~ \false                    & \text{boolean literals} \\\\
                    &   | & \rexpr : \rtype                     & \text{term annotated with a type} \\\\
                    &   | & \Pi{\var:\rtype_1}{\rtype_2}        & \text{dependent function type} \\\\
                    &   | & \lam{\var:\rtype}{\rexpr}           & \text{functions} \\\\
                    &   | & \app{\rexpr_1}{\rexpr_2}            & \text{function application} \\\\
                    &   | & \ifte{\rexpr_1}{\rexpr_2}{\rexpr_3} & \text{if expressions} \\\\
                    &   | & \case{\rexpr}{\overline{\rpat_i \rightarrow \rexpr_i}^{;}}
                                                                & \text{case expressions} \\\\
                    &   | & \Record{\label[\var]:\rtype_1, \rtype_2}
                                                                & \text{record type extension} \\\\
                    &   | & \Record{}                           & \text{empty record type} \\\\
                    &   | & \record{\label=\rexpr_1, \rexpr_2}  & \text{record extension} \\\\
                    &   | & \record{}                           & \text{empty record} \\\\
                    &   | & \rexpr.\label                       & \text{record projection} \\\\
    \\\\
    \rpat           & ::= & \var                                & \text{binder pattern} \\\\
                    &   | & \rpat : \rtype                      & \text{pattern annotated with a type} \\\\
                    &   | & \true ~|~ \false                    & \text{boolean literal patterns} \\\\
                %   &   | & \record{\label=\rpat_1, \rpat_2}    & \text{record extension pattern} \\\\
                %   &   | & \record{}                           & \text{empty record pattern} \\\\
    \\\\
\end{array}
\\]

\\[
\begin{array}{lrll}
    \Arrow{\rtype_1}{\rtype_2} & := & \Pi{\var:\rtype_1}{\rtype_2} & \text{non-dependent function types} \\\\
    \lam{\var}{\rexpr}         & := & \lam{\var:?}{\rexpr}         & \text{functions (without an annotation)} \\\\
\end{array}
\\]

### Terms

The core term syntax skips holes, ensuring that everything is fully elaborated:

\\[
\begin{array}{rrll}
    \texpr,\ttype   & ::= & \var                                & \text{variables} \\\\
                    &   | & \Type_i                             & \text{universe of types ($i \in \mathbb{N}$)} \\\\
                    &   | & \Bool                               & \text{type of booleans} \\\\
                    &   | & \true ~|~ \false                    & \text{boolean literals} \\\\
                    &   | & \texpr : \ttype                     & \text{term annotated with a type} \\\\
                    &   | & \Pi{\var:\ttype_1}{\ttype_2}        & \text{dependent function type} \\\\
                    &   | & \lam{\var:\ttype}{\texpr}           & \text{functions} \\\\
                    &   | & \app{\texpr_1}{\texpr_2}            & \text{function application} \\\\
                    &   | & \ifte{\texpr_1}{\texpr_2}{\texpr_3} & \text{if expressions} \\\\
                    &   | & \case{\texpr}{\overline{\tpat_i \rightarrow \texpr_i}^{;}}
                                                                & \text{case expressions} \\\\
                    &   | & \Record{\label[\var]:\ttype_1, \ttype_2}
                                                                & \text{record type extension} \\\\
                    &   | & \Record{}                           & \text{empty record type} \\\\
                    &   | & \record{\label=\texpr_1, \texpr_2}  & \text{record extension} \\\\
                    &   | & \record{}                           & \text{empty record} \\\\
                    &   | & \texpr.\label                       & \text{record projection} \\\\
    \\\\
    \tpat           & ::= & \var                                 & \text{binder pattern} \\\\
                    &   | & \tpat : \ttype                      & \text{pattern annotated with a type} \\\\
                    &   | & \true ~|~ \false                    & \text{boolean literal patterns} \\\\
                    &   | & \record{\label=\tpat_1, \tpat_2}    & \text{record extension pattern} \\\\
                    &   | & \record{}                           & \text{empty record pattern} \\\\
    \\\\
\end{array}
\\]

### Values

In order to make it clear what is 'stuck' and what still needs to be evaluated,
we separate our syntax into [weak head normal forms][whnf-wikipedia] (\\(\wexpr\\)),
and neutral terms (\\(\nexpr\\)):

\\[
\begin{array}{rrll}
    \vexpr,\vtype   & ::= & \wexpr                              & \text{weak head normal forms} \\\\
                    &   | & \nexpr                              & \text{neutral terms} \\\\
    \\\\
    \nexpr,\ntype   & ::= & \var                                & \text{variables} \\\\
                    &   | & \app{\nexpr}{\texpr}                & \text{function application} \\\\
                    &   | & \ifte{\nexpr_1}{\texpr_2}{\texpr_3} & \text{if expressions} \\\\
                    &   | & \case{\nexpr}{\overline{\tpat_i \rightarrow \texpr_i}^{;}}
                                                                & \text{case expressions} \\\\
                    &   | & \nexpr.\label                       & \text{record projection} \\\\
    \\\\
    \wexpr,\wtype   & ::= & \Type_i                             & \text{universe of types ($i \in \mathbb{N}$)} \\\\
                    &   | & \Bool                               & \text{type of booleans} \\\\
                    &   | & \true ~|~ \false                    & \text{boolean literals} \\\\
                    &   | & \Pi{\var:\vtype_1}{\vtype_2}        & \text{dependent function type} \\\\
                    &   | & \lam{\var:\vtype}{\vexpr}           & \text{functions} \\\\
                    &   | & \Record{\label[\var]:\vtype_1, \vtype_2}
                                                                & \text{record type extension} \\\\
                    &   | & \Record{}                           & \text{empty record type} \\\\
                    &   | & \record{\label=\vexpr_1, \vexpr_2}  & \text{record extension} \\\\
                    &   | & \record{}                           & \text{empty record} \\\\
    \\\\
\end{array}
\\]

[whnf-wikipedia]: https://en.wikipedia.org/wiki/Lambda_calculus_definition#Weak_head_normal_form

### Contexts

As we type check terms, we'll be passing over bindings like lambdas and pi types.
Contexts allow us to keep track of the bound parameters,
even though we don't know the exact values these will eventually take during normalization.

\\[
\begin{array}{rrll}
    \ctx    & ::= & \emptyCtx                                   & \text{the empty context} \\\\
            &   | & \extendCtx{\ctx}{\declItem{\var}{\vtype}}   & \text{context extended with a declaration} \\\\
            &   | & \extendCtx{\ctx}{\defnItem{\var}{\texpr}}   & \text{context extended with a definition} \\\\
\end{array}
\\]

## Semantics

We take a _bidirectional_ approach to type checking, splitting it into two
phases: type checking and type inference. This makes the flow of information
through the type checker clear and relatively easy to reason about.
Normalization happens after inference, and before types are fed back in to be
used during type checking.

With that in mind, the next sections will describe the following judgments:

| Name                                                      | Notation                                                      | Inputs                                    | Outputs                                   |
|-----------------------------------------------------------|---------------------------------------------------------------|-------------------------------------------|-------------------------------------------|
| [normalization](#normalization)                           | \\(\eval{ \ctx }{ \texpr }{ \vexpr }\\)                       | \\(\ctx\\), \\(\rexpr\\)                  | \\(\vexpr\\)                              |
| [type checking](#type-checking)                           | \\(\check{ \ctx }{ \rexpr }{ \vtype }{ \texpr }\\)            | \\(\ctx\\), \\(\rexpr\\), \\(\vtype\\)    | \\(\texpr\\)                              |
| [type inference](#type-inference)                         | \\(\infer{ \ctx }{ \rexpr }{ \vtype }{ \texpr }\\)            | \\(\ctx\\), \\(\rexpr\\)                  | \\(\vtype\\), \\(\texpr\\)                |
| [pattern matching](#pattern-matching)                     | \\(\match{ \wexpr }{ \tpat }{ \theta }\\)                     | \\(\wexpr\\), \\(\tpat\\)                 | \\(\theta\\)                              |
| [type checking of patterns](#type-checking-of-patterns)   | \\(\checkpat{ \ctx }{ \rpat }{ \vtype }{ \tpat }{ \ctx' }\\)  | \\(\ctx\\), \\(\rpat\\), \\(\vtype\\)     | \\(\tpat\\), \\(\ctx'\\)                  |
| [type inference of patterns](#type-inference-of-patterns) | \\(\inferpat{ \ctx }{ \rpat }{ \vtype }{ \tpat }{ \ctx' }\\)  | \\(\ctx\\), \\(\rpat\\),                  | \\(\vtype\\), \\(\tpat\\), \\(\ctx'\\)    |

Normalization stands on its own, but both checking and inference are mutually
dependent on each other. Care has been taken to design the judgments so that
they are _syntax-directed_, meaning that an algorithm can be clearly derived
from them.

### Elaboration

Elaboration is the process of filling in missing information that the
programmer omitted in the original code, generally based on the results
of type inference.

In Pikelet's judgements the elaborated terms are denoted after the
diamond: \\(\rhd\\). At the moment not much is added - only the missing
type annotations on function parameters. In the future this could be extended
filling in type class instances and implicit arguments.

### Normalization

Here we describe how we normalize elaborated terms under the assumptions
in the context.

\\[
\boxed{
    \eval{ \ctx }{ \texpr }{ \vexpr }
}
\\\\[2em]
\begin{array}{cl}
    \rule{E-ANN}{
        \eval{ \ctx }{ \texpr }{ \vexpr }
    }{
        \eval{ \ctx }{ \texpr:\ttype }{ \vexpr }
    }
    \\\\[2em]
    \rule{E-TYPE}{}{
        \eval{ \ctx }{ \Type_i }{ \Type_i }
    }
    \\\\[2em]
    \rule{E-BOOL}{}{
        \eval{ \ctx }{ \Bool }{ \Bool }
    }
    \\\\[2em]
    \rule{E-TRUE}{}{
        \eval{ \ctx }{ \true }{ \true }
    }
    \\\\[2em]
    \rule{E-FALSE}{}{
        \eval{ \ctx }{ \false }{ \false }
    }
    \\\\[2em]
    \rule{E-VAR}{
        \defnItem{\var}{\texpr} \notin \ctx
    }{
        \eval{ \ctx }{ \var }{ \var }
    }
    \\\\[2em]
    \rule{E-VAR-DEF}{
        \defnItem{\var,\texpr} \in \ctx
        \qquad
        \eval{ \ctx }{ \texpr }{ \vexpr }
    }{
        \eval{ \ctx }{ \var }{ \vexpr }
    }
    \\\\[2em]
    \rule{E-PI}{
        \eval{ \ctx }{ \ttype_1 }{ \vtype_1 }
        \qquad
        \eval{ \ctx }{ \ttype_2 }{ \vtype_2 }
    }{
        \eval{ \ctx }{ \Pi{\var:\ttype_1}{\ttype_2} }{ \Pi{\var:\vtype_1}{\vtype_2} }
    }
    \\\\[2em]
    \rule{E-LAM}{
        \eval{ \ctx }{ \ttype }{ \vtype }
        \qquad
        \eval{ \ctx }{ \texpr }{ \vexpr }
    }{
        \eval{ \ctx }{ \lam{\var:\ttype}{\texpr} }{ \lam{\var:\vtype}{\vexpr} }
    }
    \\\\[2em]
    \rule{E-APP}{
        \eval{ \ctx }{ \texpr_1 }{ \lam{\var:\vtype_1}{\vexpr_1} }
        \qquad
        \eval{ \ctx }{ \subst{\vexpr_1}{\var}{\texpr_2} }{ \vexpr_3 }
    }{
        \eval{ \ctx }{ \app{\texpr_1}{\texpr_2} }{ \vexpr_3 }
    }
    \\\\[2em]
    \rule{E-IF}{
        \eval{ \ctx }{ \nexpr }{ \nexpr' }
    }{
        \eval{ \ctx }{ \ifte{\nexpr}{\texpr_1}{\texpr_2} }{ \ifte{\nexpr'}{\texpr_1}{\texpr_2} }
    }
    \\\\[2em]
    \rule{E-IF-TRUE}{
        \eval{ \ctx }{ \nexpr }{ \true }
        \qquad
        \eval{ \ctx }{ \texpr_1 }{ \vexpr_1 }
    }{
        \eval{ \ctx }{ \ifte{\nexpr}{\texpr_1}{\texpr_2} }{ \vexpr_1 }
    }
    \\\\[2em]
    \rule{E-IF-FALSE}{
        \eval{ \ctx }{ \nexpr }{ \false }
        \qquad
        \eval{ \ctx }{ \texpr_2 }{ \vexpr_2 }
    }{
        \eval{ \ctx }{ \ifte{\nexpr}{\texpr_1}{\texpr_2} }{ \vexpr_2 }
    }
    \\\\[2em]
    \rule{E-CASE}{
        \eval{ \ctx }{ \nexpr }{ \nexpr' }
    }{
        \eval{ \ctx }{ \case{\nexpr}{\overline{\tpat_i \rightarrow \texpr_i}^{;}} }
            { \case{\nexpr'}{\overline{\tpat_i \rightarrow \texpr_i}^{;}} }
    }
    \\\\[2em]
    \rule{E-CASE-MATCH}{
        \eval{ \ctx }{ \nexpr }{ \wexpr }
        \qquad
        \match{ \wexpr }{ \tpat_i }{ \theta }
        \qquad
        \eval{ \ctx }{ \texpr_i ~ \theta }{ \vexpr_i }
    }{
        \eval{ \ctx }{ \case{\nexpr}{\overline{\tpat_i \rightarrow \texpr_i}^{;}} }{ \vexpr_i }
    }
    \\\\[2em]
    \rule{E-RECORD-TYPE}{
        \eval{ \ctx }{ \ttype_1 }{ \vtype_1 }
        \qquad
        \eval{ \ctx }{ \ttype_2 }{ \vtype_2 }
    }{
        \eval{ \ctx }{ \Record{\label[\var]:\ttype_1, \ttype_2} }{ \Record{\label[\var]:\vtype_1, \vtype_2} }
    }
    \\\\[2em]
    \rule{E-RECORD}{
        \eval{ \ctx }{ \texpr_1 }{ \vexpr_1 }
        \qquad
        \eval{ \ctx }{ \texpr_2 }{ \vexpr_2 }
    }{
        \eval{ \ctx }{ \record{\label=\texpr_1, \texpr_2} }{ \record{\label=\vexpr_1, \vexpr_2} }
    }
    \\\\[2em]
    \rule{E-EMPTY-RECORD-TYPE}{}{
        \eval{ \ctx }{ \Record{} }{ \Record{} }
    }
    \\\\[2em]
    \rule{E-EMPTY-RECORD}{}{
        \eval{ \ctx }{ \record{} }{ \record{} }
    }
    \\\\[2em]
    \rule{E-PROJ}{
        \eval{ \ctx }{ \texpr_1 }{ \vexpr_1 }
        \qquad
        \vexpr_2 = \field(\label, \vexpr_1)
    }{
        \eval{ \ctx }{ \texpr_1.\label }{ \vexpr_2 }
    }
    \\\\[2em]
\end{array}
\\]

We define \\(\field(-,-)\\) like so:

\\[
\begin{array}{lrll}
    \field(\label_1, \record{\label_2 = \vexpr_1, \vexpr_2}) & = & \vexpr_1 & \text{if} ~ \label_1 \equiv \label_2 \\\\
    \field(\label_1, \record{\label_2 = \vexpr_1, \vexpr_2}) & = & \field(\label_1, \vexpr_2) \\\\
\end{array}
\\]

### Type checking

This judgement checks that the given term has the expected type and returns its
elaborated form.

\\[
\boxed{
    \check{ \ctx }{ \rexpr }{ \vtype }{ \texpr }
}
\\\\[2em]
\begin{array}{cl}
    \rule{C-LAM}{
        \infer{ \extendCtx{\ctx}{\declItem{\var}{\vtype_1}} }{ \rexpr }{ \ttype_2 }{ \texpr }
    }{
        \check{ \ctx }{ \lam{\var}{\rexpr} }{ \Pi{\var:\vtype_1}{\vtype_2} }{ \lam{\var:\vtype_1}{\texpr} }
    }
    \\\\[2em]
    \rule{C-IF}{
        \check{ \ctx }{ \rexpr_1 }{ \Bool }{ \texpr_1 }
        \qquad
        \check{ \ctx }{ \rexpr_2 }{ \vtype }{ \texpr_2 }
        \qquad
        \check{ \ctx }{ \rexpr_3 }{ \vtype }{ \texpr_3 }
    }{
        \check{ \ctx }{ \ifte{\rexpr_1}{\rexpr_2}{\rexpr_3} }{ \vtype }{ \ifte{\texpr_1}{\texpr_2}{\texpr_3} }
    }
    \\\\[2em]
    \rule{C-CASE}{
        \infer{ \ctx }{ \rexpr }{ \vtype_1 }{ \texpr }
        \qquad
        \overline{
            % TODO: impl pattern checks
            ~
            \check{ \ctx }{ \rpat_i }{ \vtype_1 }{ \tpat_i } \Rightarrow \ctx'
            \qquad
            \check{ \composeCtx{\ctx}{\ctx'} }{ \rexpr_i }{ \vtype_2 }{ \texpr_i }
            ~
        }
    }{
        \check{ \ctx }{ \case{\rexpr}{\overline{\rpat_i \rightarrow \rexpr_i}^{;}} }{ \vtype_2 }
            { \case{\texpr}{\overline{\tpat_i \rightarrow \texpr_i}^{;}} }
    }
    \\\\[2em]
    \rule{C-RECORD}{
        \label_1 \equiv \label_2
        \qquad
        \check{ \ctx }{ \rexpr_1 }{ \vtype_1 }{ \texpr_1 }
        \qquad
        \eval{ \ctx }{ \subst{\vtype_2}{\var}{\texpr_1} }{ \vtype_3 }
        \qquad
        \check{ \ctx }{ \rexpr_2 }{ \vtype_3 }{ \texpr_2 }
    }{
        \check{ \ctx }{ \record{\label_1=\rexpr_1, \rexpr_2} }
            { \Record{\label_2[\var]:\vtype_1, \vtype_2} }
            { \record{\label_1=\texpr_1, \texpr_2} }
    }
    \\\\[2em]
    \rule{C-CONV}{
        \infer{ \ctx }{ \rexpr }{ \vtype_2 }{ \texpr }
        \qquad
        \vtype_1 \equiv_{\alpha} \vtype_2
    }{
        \check{ \ctx }{ \rexpr }{ \vtype_1 }{ \texpr }
    }
    \\\\[2em]
\end{array}
\\]

In C-CONV we flip the direction of the type checker, comparing the type of the
expected term for [alpha equivalence] with the inferred term. Note that we could
alternatively check for subtyping instead of alpha equivalence. This could be
useful for implementing a cumulative universe hierarchy.

[alpha equivalence]: https://en.wikipedia.org/wiki/Lambda_calculus#Alpha_equivalence

### Type inference

Here we define a judgement that synthesizes a type from the given term and
returns its elaborated form.

\\[
\boxed{
    \infer{ \ctx }{ \rexpr }{ \vtype }{ \texpr }
}
\\\\[2em]
\begin{array}{cl}
    \rule{I-ANN}{
        \infer{ \ctx }{ \rtype }{ \Type_i }{ \ttype }
        \qquad
        \eval{ \ctx }{ \ttype }{ \vtype }
        \qquad
        \check{ \ctx }{ \rexpr }{ \vtype }{ \texpr }
    }{
        \infer{ \ctx }{ \rexpr:\rtype }{ \Type_{i+1} }{ \texpr:\ttype }
    }
    \\\\[2em]
    \rule{I-TYPE}{}{
        \infer{ \ctx }{ \Type_i }{ \Type_{i+1} }{ \Type_i }
    }
    \\\\[2em]
    \rule{I-BOOL}{}{
        \infer{ \ctx }{ \Bool }{ \Type_0 }{ \Bool }
    }
    \\\\[2em]
    \rule{I-TRUE}{}{
        \infer{ \ctx }{ \true }{ \Bool }{ \true }
    }
    \\\\[2em]
    \rule{I-FALSE}{}{
        \infer{ \ctx }{ \false }{ \Bool }{ \false }
    }
    \\\\[2em]
    \rule{I-VAR}{
        \declItem{\var}{\vtype} \in \ctx
    }{
        \infer{ \ctx }{ \var }{ \vtype }{ \var }
    }
    \\\\[2em]
    \rule{I-PI}{
        \infer{ \ctx }{ \rtype_1 }{ \Type_i }{ \ttype_1 }
        \qquad
        \eval{ \ctx }{ \ttype_1 }{ \vtype_1 }
        \qquad
        \check{ \extendCtx{\ctx}{\declItem{\var}{\vtype_1}} }{ \rtype_2 }{ \Type_j }{ \ttype_2 }
    }{
        \infer{ \ctx }{ \Pi{\var:\rtype_1}{\rtype_2} }{ \Type_{\max(i,j)} }{ \Pi{\var:\ttype_1}{\ttype_2} }
    }
    \\\\[2em]
    \rule{I-LAM}{
        \infer{ \ctx }{ \rtype }{ \Type_i }{ \ttype }
        \qquad
        \eval{ \ctx }{ \ttype }{ \vtype_1 }
        \qquad
        \check{ \extendCtx{\ctx}{\declItem{\var}{\vtype_1}} }{ \rexpr}{ \vtype_2 }{ \texpr }
    }{
        \infer{ \ctx }{ \lam{\var:\rtype}{\rexpr} }{ \Pi{\var:\vtype_1}{\vtype_2} }{ \lam{\var:\ttype}{\texpr} }
    }
    \\\\[2em]
    \rule{I-APP}{
        \infer{ \ctx }{ \rexpr_1 }{ \Pi{\var:\vtype_1}{\vtype_2} }{ \texpr_1 }
        \qquad
        \check{ \ctx }{ \rexpr_2 }{ \vtype_1 }{ \texpr_2 }
        \qquad
        \eval{ \ctx }{ \subst{\vtype_2}{\var}{\texpr_2} }{ \vtype_3 }
    }{
        \infer{ \ctx }{ \app{\rexpr_1}{\rexpr_2} }{ \vtype_3 }{ \app{\texpr_1}{\texpr_2} }
    }
    \\\\[2em]
    \rule{I-RECORD-TYPE}{
        \infer{ \ctx }{ \rtype_1 }{ \Type_i }{ \ttype_1 }
        \qquad
        \eval{ \ctx }{ \ttype_1 }{ \vtype_1 }
        \qquad
        \infer{ \extendCtx{\ctx}{\declItem{\var}{\vtype_1}} }{ \rtype_2 }{ \Type_j }{ \ttype_2 }
    }{
        \infer{ \ctx }
            { \Record{\label[\var]:\rtype_1, \rtype_2} }
            { \Type_{\max(i,j)} }
            { \Record{\label[\var]:\ttype_1, \ttype_2} }
    }
    \\\\[2em]
    \rule{I-EMPTY-RECORD-TYPE}{}{
        \infer{ \ctx }{ \Record{} }{ \Type_0 }{ \Record{} }
    }
    \\\\[2em]
    \rule{I-EMPTY-RECORD}{}{
        \infer{ \ctx }{ \record{} }{ \Record{} }{ \record{} }
    }
    \\\\[2em]
    \rule{I-PROJ}{
        \infer{ \ctx }{ \rexpr }{ \vtype_1 }{ \texpr }
        \qquad
        \vtype_2 = \fieldty(\label, \vtype_1)
        \qquad
        \theta = \fieldsubst(\texpr, \label, \vtype_1)
    }{
        \infer{ \ctx }{ \rexpr.\label }{ \vtype_2 ~ \theta }{ \texpr.\label }
    }
    \\\\[2em]
\end{array}
\\]

We define \\(\fieldty(-,-)\\) and \\(\fieldsubst(-,-,-)\\) like so:

\\[
\begin{array}{lrll}
    \fieldty(\label_1, \Record{\label_2 : \vtype_1, \vtype_2}) & = & \vtype_1 & \text{if} ~ \label_1 \equiv \label_2 \\\\
    \fieldty(\label_1, \Record{\label_2 : \vtype_1, \vtype_2}) & = & \fieldty(\label_1, \vtype_2) \\\\
    \\\\[2em]
\end{array}
\\]

In order to ensure that we maintain maintain the proper paths to variables when
we project on them, we define \\(\fieldsubst(-,-,-)\\) as:

\\[
\begin{array}{lrll}
    \fieldsubst(\texpr, \label_1, \Record{\label_2 : \vtype_1, \vtype_2}) & =
        & [] & \text{if} ~ \label_1 \equiv \label_2 \\\\
    \fieldsubst(\texpr, \label_1, \Record{\label_2 : \vtype_1, \vtype_2}) & =
        & \fieldsubst(\texpr, \label_1, \vtype_2) \doubleplus [ \label_2 \rightarrow \texpr.\label_2 ] \\\\
    \\\\[2em]
\end{array}
\\]

### Pattern matching

This judement takes an expression \\(\wexpr\\) in weak head normal form, and a
pattern \\(\tpat\\) and returns a substitution \\(\theta\\) with the matched bindings.

\\[
\boxed{
    \match{ \wexpr }{ \tpat }{ \theta }
}
\\\\[2em]
\begin{array}{cl}
    \rule{M-VAR}{}{
        \match{ \wexpr }{ \var }{ [\var \rightarrow \wexpr] }
    }
    \\\\[2em]
    \rule{M-TRUE}{}{
        \match{ \true }{ \true }{ [] }
    }
    \\\\[2em]
    \rule{M-FALSE}{}{
        \match{ \false }{ \false }{ [] }
    }
    \\\\[2em]
% TODO:
%   \rule{M-RECORD}{
%       \match{ \wexpr_1 }{ \tpat_1 }{ \theta_1 }
%       \qquad
%       \match{ \wexpr_2 }{ \tpat_2 }{ \theta_2 }
%   }{
%       \match{ \record{\label=\wexpr_1, \wexpr_2} }{ \record{\label=\tpat_1, \tpat_2} }{ \theta_1 \doubleplus \theta_2 }
%   }
%   \\\\[2em]
%   \rule{M-EMPTY-RECORD}{}{
%       \match{ \record{} }{ \record{} }{ [] }
%   }
%   \\\\[2em]
\end{array}
\\]

### Type checking of patterns

\\[
\boxed{
    \checkpat{ \ctx }{ \rpat }{ \vtype }{ \tpat }{ \ctx' }
}
\\\\[2em]
\begin{array}{cl}
    \rule{CP-BINDER}{}{
        \checkpat{ \ctx }{ \var }{ \vtype }{ \var }{ \var : \vtype }
    }
    \\\\[2em]
    \rule{CP-CONV}{
        \inferpat{ \ctx }{ \rpat }{ \vtype_2 }{ \tpat }{ \ctx' }
        \qquad
        \vtype_1 \equiv_{\alpha} \vtype_2
    }{
        \checkpat{ \ctx }{ \rpat }{ \vtype_1 }{ \tpat }{ \ctx' }
    }
    \\\\[2em]
\end{array}
\\]

### Type inference of patterns

\\[
\boxed{
    \inferpat{ \ctx }{ \rpat }{ \vtype }{ \tpat }{ \ctx' }
}
\\\\[2em]
\begin{array}{cl}
    \rule{IP-ANN}{
        \infer{ \ctx }{ \rtype }{ \Type_i }{ \ttype }
        \qquad
        \eval{ \ctx }{ \ttype }{ \vtype }
        \qquad
        \checkpat{ \ctx }{ \rpat }{ \vtype }{ \rpat }{ \ctx' }
    }{
        \inferpat{ \ctx }{ \rpat : \rtype }{ \rtype }{ \rpat : \rtype }{ \ctx' }
    }
    \\\\[2em]
    \rule{IP-TRUE}{}{
        \inferpat{ \ctx }{ \true }{ \Bool }{ \true }{ \emptyCtx }
    }
    \\\\[2em]
    \rule{IP-FALSE}{}{
        \inferpat{ \ctx }{ \false }{ \Bool }{ \false }{ \emptyCtx }
    }
    \\\\[2em]
\end{array}
\\]

> **TODO:**
>
> - Coverage (ie. that a series of patterns covers all possible values)
> - Ensure that parametericity is maintained. Should we forbid [pattern matching
>   directly on types][type-patterns]? McBride seems to [think we can have our
>   cake and eat it][type-patterns-mcbride]!

[type-patterns]: https://stackoverflow.com/questions/45439486/pattern-matching-on-type-in-idris
[type-patterns-mcbride]: https://stackoverflow.com/questions/23220884/why-is-typecase-a-bad-thing/26012264#26012264
