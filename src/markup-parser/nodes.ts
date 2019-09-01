export abstract class Node {
  constructor(public children: Node[] = []) { }
}

export class NodeChar extends Node {
  constructor(public value: string) {
    super()
  }
}

export class NodeString extends Node {
  constructor(public value: string) {
    super()
  }
}

export class NodeBold extends Node {
  constructor(public value: string) {
    super()
  }
}

export class NodeMono extends Node {
  constructor(public value: string) {
    super()
  }
}

export class NodeStrikethrough extends Node {
  constructor(public value: string) {
    super()
  }
}

export class NodeLink extends Node {
  constructor(
    public link: string,
    public description: string,
  ) {
    super()
  }
}

type InlineNodes = NodeBold | NodeMono | NodeStrikethrough | NodeLink | NodeString

export class NodeListItem extends Node {
  constructor(public children: InlineNodes[]) {
    super()
  }
}

export class NodeUnorderedList extends Node {
  constructor(public children: NodeListItem[]) {
    super()
  }
}

export class NodeHeader extends Node {
  constructor(
    public value: string,
    public level: 1 | 2,
  ) {
    super()
  }
}

export class NodeCodeBlock extends Node {
  constructor(
    public lang: string,
    public value: string,
  ) {
    super()
  }
}

export class NodeParagraph extends Node {
  constructor(public children: Array<NodeHeader | NodeUnorderedList | NodeCodeBlock | InlineNodes>) {
    super()
  }
}

export class NodeNewlines extends Node { }

export class NodeMarkup extends Node {
  constructor(public children: Array<NodeNewlines | NodeParagraph>) {
    super()
  }
}
