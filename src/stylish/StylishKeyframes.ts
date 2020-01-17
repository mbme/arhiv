import {
  IStyleObject,
  StyleTransformer,
} from './types'
import { StylishRenderer } from './StylishRenderer'
import { applyTransformer } from './utils'

export class StylishKeyframes {
  constructor(
    private _keyframes: IStyleObject,
    private _renderer: StylishRenderer,
    private _transformer?: StyleTransformer,
  ) { }

  private _generateAnimationName(): string {
    return this._renderer.renderKeyframes(applyTransformer(this._keyframes, this._transformer))
  }

  private _animationName?: string

  get animationName() {
    this._animationName = this._animationName || this._generateAnimationName()

    return this._animationName
  }
}
