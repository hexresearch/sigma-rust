import { expect, assert } from 'chai';

import {
  ErgoTree,
  Constant
} from '../pkg/ergo_lib_wasm';

it('constants_len', async () => {
  let tree_bytes_base16_str = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree != null);
  assert(tree.constants_len() == 2);
});

it('get_constant', async () => {
  let tree_bytes_base16_str = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree != null);
  assert(tree.constants_len() == 2);
  assert(tree.get_constant(0) != null);
  assert(tree.get_constant(1) != null);
});

it('get_constant out of bounds', async () => {
  let tree_bytes_base16_str = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree != null);
  assert(tree.constants_len() == 2);
  assert(tree.get_constant(3) == null);
});

it('set_constant', async () => {
  let tree_bytes_base16_str = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree.constants_len() == 2);
  let constant = Constant.from_i32(99);
  assert(tree.set_constant(0, constant) != null);
  assert(tree.get_constant(0).to_i32() == 99);
});

it('set_constant out of bounds', async () => {
  let tree_bytes_base16_str = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree.constants_len() == 2);
  let constant = Constant.from_i32(99);
  assert(tree.set_constant(3, constant) == null);
});
