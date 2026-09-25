import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  weight: ["300", "400", "500", "600", "700"],
  variable: "--font-inter",
});

export const metadata: Metadata = {
  title: "AIOS · laya-gateway",
  description:
    "Local gateway dashboard — Laya decides the tool; the LLM does the rest",
  icons: {
    icon: "/img/logo/aios.jpeg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" className="light">
      <body className={`${inter.variable} antialiased`}>{children}</body>
    </html>
  );
}
