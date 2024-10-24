#!/usr/bin/env zx
import 'zx/globals';
import * as c from 'codama';
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';

// Instanciate Codama from the IDL.
const idl = require(path.join(__dirname, '..', 'api', 'idl.json'));
const codama = c.createFromRoot(rootNodeFromAnchor(idl));

// Rename the program.
codama.update(
  c.updateProgramsVisitor({
    oreApi: { name: 'ore' },
  })
);

// Transform the `Treasury` defined type into and account. This is currenttly
// necessary because the `Treasury` is an empty struct and Shank does not export
// empty structs as accounts.
codama.update(c.transformDefinedTypesIntoAccountsVisitor(['treasury']));

codama.update(
  c.bottomUpTransformerVisitor([
    // Associate instrution data with the instruction node. This takes
    // advantage of the fact that the instruction name is the same as the
    // instruction data struct name.
    {
      select: '[instructionNode]',
      transform: (node, stack) => {
        c.assertIsNode(node, 'instructionNode');
        const program = stack.getProgram();
        c.assertIsNode(program, 'programNode');
        const data = c
          .getAllDefinedTypes(program)
          .find((definedType) => definedType.name === node.name);
        c.assertIsNode(data.type, 'structTypeNode');

        if (data.type.fields.length === 0) {
          return node;
        }

        const args = data.type.fields.map((field) => {
          return c.instructionArgumentNode({ ...field });
        });

        return c.instructionNode({
          ...node,
          arguments: [...node.arguments, ...args],
        });
      },
    },
    // Adds a discriminator field to each account since the discriminator
    // is not 'visible' from the account struct.
    {
      select: '[accountNode]',
      transform: (node, stack) => {
        c.assertIsNode(node, 'accountNode');
        return c.accountNode({
          ...node,
          data: c.transformNestedTypeNode(node.data, (struct) =>
            c.structTypeNode([
              c.structFieldTypeNode({
                name: 'discriminator',
                type: c.numberTypeNode('u64'),
              }),
              ...struct.fields,
            ])
          ),
        });
      },
    },
    // Renames the `OreAccount` defined type to `OreDiscriminator`. Codama generates
    // an `OreAccount` enum with account names as variants automatically.
    {
      select: '[definedTypeNode]oreAccount',
      transform: (node) => {
        c.assertIsNode(node, 'definedTypeNode');
        return c.definedTypeNode({
          ...node,
          name: 'oreDiscriminator',
        });
      },
    },
  ])
);

// Adds seeds to the PDA accounts.
codama.update(
  c.updateAccountsVisitor({
    bus: {
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'bus'),
        c.variablePdaSeedNode(
          'id',
          c.numberTypeNode('u64'),
          'The ID of the bus account.'
        ),
      ],
    },
    config: {
      seeds: [c.constantPdaSeedNodeFromString('utf8', 'config')],
    },
    proof: {
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'proof'),
        c.variablePdaSeedNode(
          'authority',
          c.publicKeyTypeNode(),
          'The signer authorized to use this proof.'
        ),
      ],
    },
    treasury: {
      seeds: [c.constantPdaSeedNodeFromString('utf8', 'treasury')],
    },
  })
);

// Set account discriminators.
codama.update(
  c.setAccountDiscriminatorFromFieldVisitor({
    bus: {
      field: 'discriminator',
      value: c.numberValueNode(100),
    },
    config: {
      field: 'discriminator',
      value: c.numberValueNode(101),
    },
    proof: {
      field: 'discriminator',
      value: c.numberValueNode(102),
    },
    treasury: {
      field: 'discriminator',
      value: c.numberValueNode(103),
    },
  })
);

// Render JavaScript.
const jsClient = path.join(__dirname, '..', 'clients', 'js');
codama.accept(
  renderJavaScriptVisitor(path.join(jsClient, 'src', 'generated'), {
    prettierOptions: require(path.join(jsClient, '.prettierrc.json')),
  })
);
