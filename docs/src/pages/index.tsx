import React from "react";
import clsx from "clsx";
import Layout from "@theme/Layout";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import styles from "./index.module.css";
import { Features, FeatureItem } from "@site/src/components/Features";

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx("hero hero--primary", styles.heroBanner)}>
      <div className="container">
        <div className={clsx("rotzname", styles.rotzname)}>
          <span>Rust Dotfilemanager</span>
          <span>Rust Dotfile manager</span>
          <span>Rust Dotfile s</span>
          <span>Rust Dot s</span>
          <span>R ust Dots</span>
          <span>R ots</span>
          <span>Rot s</span>
          <span style={{ fontWeight: 600, fontSize: "1.5em" }}>Rotz 👃</span>
        </div>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/getting-started"
          >
            Getting started
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="/docs/configuration"
          >
            Configuration
          </Link>
        </div>
      </div>
    </header>
  );
}

let commandList: FeatureItem[] = [
  {
    emoji: "⬇️",
    title: "Clone your dotfiles",
    description: (
      <>
        With Rotz you can clone your dotfiles from a git repository using the{" "}
        <code>rotz clone</code> command.
      </>
    ),
  },
  {
    emoji: "💿",
    title: "Install software",
    description: (
      <>
        You can bootstrap your new machine using the <code>rotz install</code>{" "}
        command.
      </>
    ),
  },
  {
    emoji: "🚀",
    title: "Deploy dotfiles",
    description: (
      <>
        You can automatically symlink your dotfiles to the correct places using
        the <code>rotz link</code> command.
      </>
    ),
  },
];

let featureList: FeatureItem[] = [
  {
    emoji: "⚙️",
    title: "Versatile configuration",
    description: (
      <>
        You can specify where to link your dotfiles to and what software to
        install in <code>yaml</code>, <code>toml</code> or <code>json</code>{" "}
        config files.
      </>
    ),
  },
  {
    emoji: "🪟🐧🍎",
    title: "Cross platform",
    description: (
      <>
        Rotz works on Windows, Linux and MacOs and has full support for
        different configurations on each platform.
      </>
    ),
  },
  {
    emoji: "🦀",
    title: "Open source and written in rust",
    description: (
      <>
        If you find a bug or have a feature request feel free to open a{" "}
        <a href="https://github.com/volllly/rotz/issues">github issue</a> or
        even a pull request.
      </>
    ),
  },
];

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout description="Fully cross platform dotfile manager and dev environment bootstrapper written in Rust.">
      <HomepageHeader />
      <main className={clsx("hero", styles.heroBanner)}>
        <div className="container">
          <Features features={commandList} />
        </div>
        <hr
          style={{ width: "calc(var(--ifm-container-width) / 6)", margin: 0 }}
        />
        <div className="container">
          <Features features={featureList} />
        </div>
      </main>
    </Layout>
  );
}
