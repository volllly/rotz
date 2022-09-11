import React from 'react';
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import CodeBlock from '@theme/CodeBlock';
import YAML from 'yaml';
import TOML from '@ltd/j-toml';

export type TabedCodeBlockProps = {
  title: string;
  data?: any;
  yaml?: string;
  toml?: string;
  json?: string;
  predots: boolean;
};

export default function TabedCodeBlock({ title, data, yaml, toml, json, predots }: TabedCodeBlockProps): JSX.Element {
  yaml = yaml ?? YAML.stringify(data);
  toml = toml ?? TOML.stringify(data, {
    newline: "\n",
    indent: "  "
  });
  json = json ?? JSON.stringify(data, null, "  ");

  yaml = yaml.trim();
  toml = toml.trim();
  json = json.trim();

  if (predots) {
    yaml = `...\n\n${yaml}`;
    toml = `...\n\n${toml}`;
    json = `{\n...\n${json.substring(1)}`;
  }

  return (
    <Tabs>
      <TabItem value="yaml" label="yaml" default>
        <CodeBlock title={title.replace("{{ format }}", "yaml")} language="yaml">
          {yaml}
        </CodeBlock>
      </TabItem>
      <TabItem value="toml" label="toml" default>
        <CodeBlock title={title.replace("{{ format }}", "toml")} language="toml">
          {toml}
        </CodeBlock>
      </TabItem>
      <TabItem value="json" label="json" default>
        <CodeBlock title={title.replace("{{ format }}", "json")} language="json">
          {json}
        </CodeBlock>
      </TabItem>
    </Tabs>
  );
}
